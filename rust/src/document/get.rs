use super::*;

fn key_of(node: &SyntaxNode, source: &str) -> (Option<String>, Location) {
  node
    .parent()
    .and_then(|parent| {
      use yaml_parser::ast::{BlockMapEntry, FlowMapEntry};

      let key_node = match BlockMapEntry::cast(parent.clone()) {
        Some(entry) => entry.key().map(|key| key.syntax().clone()),
        None => FlowMapEntry::cast(parent).and_then(|entry| entry.key().map(|key| key.syntax().clone())),
      }?;

      let key_text = extract_scalar_text(&key_node)?;
      let key_range = key_node.text_range();
      let key_location = compute_location(source, key_range.start().into(), key_range.end().into());

      Some((key_text, key_location))
    })
    .map(|(name, location)| (Some(name), location))
    .unwrap_or((None, Location::default()))
}

impl Document {
  pub fn is_valid_selector(&self, dot_path: &str) -> bool {
    if dot_path.is_empty() {
      return true;
    }

    let selectors = self.selectors();

    if selectors.contains(&dot_path.to_string()) {
      return true;
    }

    if crate::selector::Selector::parse(dot_path).has_wildcard() && !self.navigate_all_compact(dot_path).is_empty() {
      return true;
    }

    let wildcard_version = dot_path.replace(|c: char| c.is_ascii_digit(), "").replace("[.", "[");

    let mut normalized = String::new();
    let mut chars = dot_path.chars().peekable();

    while let Some(char) = chars.next() {
      if char == '[' {
        normalized.push('[');

        while chars.peek().map(|char| char.is_ascii_digit()).unwrap_or(false) {
          chars.next();
        }
      } else {
        normalized.push(char);
      }
    }

    selectors.contains(&normalized) || selectors.contains(&wildcard_version)
  }

  pub fn get(&self, dot_path: &str) -> Option<String> {
    if dot_path.contains('[') {
      return self.get_all(dot_path).into_iter().next();
    }

    let current_node = self.navigate(dot_path).ok()?;

    extract_scalar_text(&current_node)
  }

  pub fn get_all(&self, dot_path: &str) -> Vec<String> {
    self.navigate_all_compact(dot_path).iter().filter_map(extract_scalar_text).collect()
  }

  pub fn get_typed(&self, dot_path: &str) -> Option<ScalarValue> {
    if crate::selector::Selector::parse(dot_path).has_wildcard() {
      return self.get_all_typed(dot_path).into_iter().next();
    }

    let current_node = self.navigate(dot_path).ok()?;

    if current_node.descendants().any(|child| {
      matches!(
        child.kind(),
        SyntaxKind::BLOCK_MAP | SyntaxKind::BLOCK_SEQ | SyntaxKind::FLOW_SEQ | SyntaxKind::FLOW_MAP
      )
    }) {
      return None;
    }

    extract_scalar(&current_node)
  }

  pub fn get_all_typed(&self, dot_path: &str) -> Vec<ScalarValue> {
    self
      .navigate_all_compact(dot_path)
      .iter()
      .filter(|node| {
        !node
          .descendants()
          .any(|child| child.kind() == SyntaxKind::BLOCK_MAP || child.kind() == SyntaxKind::BLOCK_SEQ)
      })
      .filter_map(extract_scalar)
      .collect()
  }

  pub fn resolve_selectors(&self, dot_path: &str) -> Vec<String> {
    self.navigate_all_compact(dot_path).iter().map(node_selector).collect()
  }

  pub fn keys_at(&self, dot_path: &str) -> Vec<String> {
    let node = if dot_path.is_empty() {
      self.root.clone()
    } else {
      match self.navigate_all_compact(dot_path).into_iter().next() {
        Some(node) => node,
        None => return Vec::new(),
      }
    };

    let collection = node.descendants().find(|descendant| {
      matches!(
        descendant.kind(),
        SyntaxKind::BLOCK_MAP | SyntaxKind::BLOCK_SEQ | SyntaxKind::FLOW_MAP | SyntaxKind::FLOW_SEQ
      )
    });

    let Some(collection) = collection else {
      return Vec::new();
    };

    if let Some(map) = BlockMap::cast(collection.clone()) {
      return map
        .entries()
        .filter_map(|entry| entry.key().and_then(|key| extract_scalar_text(key.syntax())))
        .collect();
    }

    if let Some(map) = yaml_parser::ast::FlowMap::cast(collection) {
      return map
        .entries()
        .map(|entries| {
          entries
            .entries()
            .filter_map(|entry| entry.key().and_then(|key| extract_scalar_text(key.syntax())))
            .collect()
        })
        .unwrap_or_default();
    }

    Vec::new()
  }

  pub fn get_all_located(&self, dot_path: &str) -> Vec<LocatedNode> {
    let source = self.source_text();
    let file_path = self.path.as_ref().map(|p| p.to_string_lossy().to_string());

    self
      .navigate_all_compact(dot_path)
      .iter()
      .map(|node| {
        let range = node.text_range();
        let offset: usize = range.start().into();
        let line = line_at(&source, offset);
        let location = compute_location(&source, offset, range.end().into());
        let selector = node_selector(node);
        let is_scalar = !node
          .descendants()
          .any(|child| child.kind() == SyntaxKind::BLOCK_MAP || child.kind() == SyntaxKind::BLOCK_SEQ);

        let (text, value_type) = if is_scalar {
          extract_scalar(node)
            .map(|s| (Some(s.text.clone()), Some(crate::syntax::detect_yaml_type(&s))))
            .unwrap_or((None, None))
        } else {
          (None, None)
        };

        let node_type = if is_scalar {
          "scalar"
        } else {
          match first_collection(node) {
            Some(FirstCollection::Sequence(_)) => "sequence",
            _ => "map",
          }
        };

        let (key_name, key_location) = key_of(node, &source);

        LocatedNode {
          node_type: node_type.to_string(),
          text,
          value_type,
          file_path: file_path.clone(),
          selector,
          line,
          location,
          key_name,
          key_location,
        }
      })
      .collect()
  }

  pub fn node_type(&self, dot_path: &str) -> NodeType {
    match self.get_value(dot_path) {
      Some(yaml_serde::Value::Mapping(_)) => NodeType::Map,
      Some(yaml_serde::Value::Sequence(_)) => NodeType::Sequence,
      Some(_) => NodeType::Scalar,
      None => NodeType::NotFound,
    }
  }

  pub fn get_node_info(&self, dot_path: &str) -> NodeInfo {
    let selector = crate::selector::Selector::parse(dot_path);
    let source = self.source_text();

    let (location, key_name, key_location) = self.resolve_location(dot_path, &source);

    if selector.has_wildcard() {
      let values = self.get_all_typed(dot_path);

      return NodeInfo {
        node_type: NodeType::Sequence,
        is_list: true,
        value: None,
        list_values: values,
        location,
        key_name,
        key_location,
      };
    }

    match self.get_typed(dot_path) {
      Some(scalar) => NodeInfo {
        node_type: NodeType::Scalar,
        is_list: false,
        value: Some(scalar),
        list_values: vec![],
        location,
        key_name,
        key_location,
      },
      None => {
        let (node_type, entry_location, entry_key_name, entry_key_location) = if self.navigate(dot_path).is_err() {
          if let Some(entry_node) = self.find_entry(dot_path) {
            let range = entry_node.text_range();
            let entry_location = compute_location(&source, range.start().into(), range.end().into());

            let (node, location) = yaml_parser::ast::BlockMapEntry::cast(entry_node.clone())
              .and_then(|entry| {
                entry.key().and_then(|key_node| {
                  let key_text = extract_scalar_text(key_node.syntax())?;
                  let key_range = key_node.syntax().text_range();
                  let key_location = compute_location(&source, key_range.start().into(), key_range.end().into());

                  Some((key_text, key_location))
                })
              })
              .map(|(name, location)| (Some(name), location))
              .unwrap_or((None, Location::default()));

            (NodeType::Scalar, entry_location, node, location)
          } else {
            (self.node_type(dot_path), location, key_name, key_location)
          }
        } else {
          (self.node_type(dot_path), location, key_name, key_location)
        };

        NodeInfo {
          node_type,
          is_list: false,
          value: None,
          list_values: vec![],
          location: entry_location,
          key_name: entry_key_name,
          key_location: entry_key_location,
        }
      }
    }
  }

  fn resolve_location(&self, dot_path: &str, source: &str) -> (Location, Option<String>, Location) {
    match self.navigate(dot_path) {
      Ok(node) => {
        let range = node.text_range();
        let location = compute_location(source, range.start().into(), range.end().into());

        let (key_name, key_location) = key_of(&node, source);

        (location, key_name, key_location)
      }
      Err(_) => (Location::default(), None, Location::default()),
    }
  }

  pub fn find_items(&self, dot_path: &str, condition: Option<&str>, select: Option<&str>) -> Result<Vec<serde_json::Value>, YerbaError> {
    let values = match condition {
      Some(cond) => self.filter(dot_path, cond)?,
      None => self.get_values(dot_path),
    };

    let select_fields: Option<Vec<&str>> = select.map(|s| s.split(',').collect());

    let items = values
      .iter()
      .map(|value| match &select_fields {
        Some(fields) => {
          let mut result = serde_json::Map::new();

          for field in fields {
            let json_value = crate::json::resolve_select_field(value, field);
            let json_key = crate::json::select_field_key(field);
            result.insert(json_key, json_value);
          }

          serde_json::Value::Object(result)
        }
        None => crate::json::yaml_to_json(value),
      })
      .collect();

    Ok(items)
  }

  pub fn get_value(&self, dot_path: &str) -> Option<yaml_serde::Value> {
    if dot_path.is_empty() {
      return Some(node_to_yaml_value(&self.root));
    }

    let parsed = crate::selector::Selector::parse(dot_path);

    if parsed.has_wildcard() {
      let padded = self.navigate_all(dot_path);

      if padded.is_empty() {
        return None;
      }

      let values: Vec<yaml_serde::Value> = padded
        .iter()
        .map(|maybe_node| match maybe_node {
          Some(node) => node_to_yaml_value(node),
          None => yaml_serde::Value::Null,
        })
        .collect();

      Some(yaml_serde::Value::Sequence(values))
    } else {
      let nodes = self.navigate_all_compact(dot_path);

      if nodes.is_empty() {
        return None;
      }

      if nodes.len() == 1 {
        return Some(node_to_yaml_value(&nodes[0]));
      }

      let values: Vec<yaml_serde::Value> = nodes.iter().map(node_to_yaml_value).collect();

      Some(yaml_serde::Value::Sequence(values))
    }
  }

  pub fn get_values(&self, dot_path: &str) -> Vec<yaml_serde::Value> {
    let parsed = crate::selector::Selector::parse(dot_path);

    if parsed.has_wildcard() {
      self
        .navigate_all(dot_path)
        .iter()
        .map(|maybe_node| match maybe_node {
          Some(node) => node_to_yaml_value(node),
          None => yaml_serde::Value::Null,
        })
        .collect()
    } else {
      self.navigate_all_compact(dot_path).iter().map(node_to_yaml_value).collect()
    }
  }

  pub fn selectors(&self) -> Vec<String> {
    let Some(value) = self.get_value("") else {
      return Vec::new();
    };

    let mut selectors = Vec::new();

    collect_selectors(&value, "", &mut selectors);
    selectors.sort();
    selectors.dedup();

    selectors
  }

  pub fn exists(&self, dot_path: &str) -> bool {
    if crate::selector::Selector::parse(dot_path).has_wildcard() {
      if !self.navigate_all_compact(dot_path).is_empty() {
        return true;
      }
    } else if self.navigate(dot_path).is_ok() {
      return true;
    }

    self.find_entry(dot_path).is_some()
  }

  fn find_entry(&self, dot_path: &str) -> Option<SyntaxNode> {
    let (parent_path, last_key) = match dot_path.rsplit_once('.') {
      Some((parent, key)) => (parent, key),
      None => ("", dot_path),
    };

    let parent_node = if parent_path.is_empty() {
      let root = Root::cast(self.root.clone())?;
      let document = root.documents().next()?;
      document.syntax().clone()
    } else {
      self.navigate(parent_path).ok()?
    };

    let map = find_block_map(&parent_node)?;

    find_entry_by_key(&map, last_key).map(|entry| entry.syntax().clone())
  }

  pub fn get_sequence_values(&self, dot_path: &str) -> Vec<String> {
    let current_node = match self.navigate(dot_path) {
      Ok(node) => node,
      Err(_) => return Vec::new(),
    };

    let sequence = match find_block_sequence(&current_node) {
      Some(sequence) => sequence,
      None => return Vec::new(),
    };

    sequence
      .entries()
      .filter_map(|entry| entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())))
      .collect()
  }

  pub fn get_collection_style(&self, dot_path: &str) -> Option<&'static str> {
    let current_node = self.navigate(dot_path).ok()?;

    for descendant in current_node.descendants() {
      match descendant.kind() {
        SyntaxKind::FLOW_SEQ | SyntaxKind::FLOW_MAP => return Some("flow"),
        SyntaxKind::BLOCK_SEQ | SyntaxKind::BLOCK_MAP => return Some("block"),
        _ => {}
      }
    }

    None
  }

  pub fn get_sequence_indent(&self, dot_path: &str) -> Option<&'static str> {
    let current_node = self.navigate(dot_path).ok()?;

    let sequence = find_block_sequence(&current_node)?;
    let first_entry = sequence.entries().next()?;

    let entry_indent = preceding_whitespace_indent(first_entry.syntax());

    let entry_node = current_node.ancestors().find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY)?;

    let source = self.source_text();
    let entry_start: usize = entry_node.text_range().start().into();
    let key_indent = &source[line_start_at(&source, entry_start)..entry_start];

    if entry_indent.len() > key_indent.len() {
      Some("indented")
    } else {
      Some("compact")
    }
  }

  pub fn get_quote_style(&self, dot_path: &str) -> Option<&'static str> {
    let current_node = self.navigate(dot_path).ok()?;

    for element in current_node.descendants_with_tokens() {
      match element.kind() {
        SyntaxKind::PLAIN_SCALAR => return Some("plain"),
        SyntaxKind::SINGLE_QUOTED_SCALAR => return Some("single"),
        SyntaxKind::DOUBLE_QUOTED_SCALAR => return Some("double"),

        SyntaxKind::BLOCK_SCALAR => {
          let text = element.as_node()?.text().to_string();
          let header = text.lines().next().unwrap_or("").trim();

          return match header {
            "|-" => Some("literal"),
            "|" => Some("literal-clip"),
            "|+" => Some("literal-keep"),
            ">-" => Some("folded"),
            ">" => Some("folded-clip"),
            ">+" => Some("folded-keep"),
            _ => None,
          };
        }

        _ => {}
      }
    }

    None
  }
}

pub(crate) fn node_selector(node: &SyntaxNode) -> String {
  let mut parts: Vec<String> = Vec::new();
  let mut current = node.clone();

  if matches!(current.kind(), SyntaxKind::BLOCK_SEQ_ENTRY | SyntaxKind::FLOW_SEQ_ENTRY) {
    if let Some(parent) = current.parent() {
      let kind = current.kind();

      let index = parent
        .children()
        .filter(|child| child.kind() == kind)
        .position(|child| child == current)
        .unwrap_or(0);

      parts.push(format!("[{}]", index));
    }
  }

  loop {
    let parent = match current.parent() {
      Some(parent) => parent,
      None => break,
    };

    match parent.kind() {
      SyntaxKind::BLOCK_SEQ_ENTRY | SyntaxKind::FLOW_SEQ_ENTRY => {
        if let Some(grandparent) = parent.parent() {
          let index = grandparent
            .children()
            .filter(|child| child.kind() == parent.kind())
            .position(|child| child == parent)
            .unwrap_or(0);

          parts.push(format!("[{}]", index));
        }
      }

      SyntaxKind::BLOCK_MAP_ENTRY => {
        if let Some(key_node) = parent.children().find(|child| child.kind() == SyntaxKind::BLOCK_MAP_KEY) {
          if let Some(key_text) = extract_scalar_text(&key_node) {
            parts.push(key_text);
          }
        }
      }

      SyntaxKind::FLOW_MAP_ENTRY => {
        if let Some(key_node) = parent.children().find(|child| child.kind() == SyntaxKind::FLOW_MAP_KEY) {
          if let Some(key_text) = extract_scalar_text(&key_node) {
            parts.push(key_text);
          }
        }
      }

      _ => {}
    }

    current = parent;
  }

  parts.reverse();
  let mut result = String::new();

  for part in &parts {
    if !part.starts_with('[') && !result.is_empty() {
      result.push('.');
    }

    result.push_str(part);
  }

  result
}
