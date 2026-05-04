use super::*;

impl Document {
  pub fn get(&self, dot_path: &str) -> Option<String> {
    if dot_path.contains('[') {
      return self.get_all(dot_path).into_iter().next();
    }

    let current_node = self.navigate(dot_path).ok()?;

    extract_scalar_text(&current_node)
  }

  pub fn get_all(&self, dot_path: &str) -> Vec<String> {
    self.navigate_all(dot_path).iter().filter_map(extract_scalar_text).collect()
  }

  pub fn get_typed(&self, dot_path: &str) -> Option<ScalarValue> {
    if crate::selector::Selector::parse(dot_path).has_wildcard() {
      return self.get_all_typed(dot_path).into_iter().next();
    }

    let current_node = self.navigate(dot_path).ok()?;

    if current_node
      .descendants()
      .any(|child| child.kind() == SyntaxKind::BLOCK_MAP || child.kind() == SyntaxKind::BLOCK_SEQ)
    {
      return None;
    }

    extract_scalar(&current_node)
  }

  pub fn get_all_typed(&self, dot_path: &str) -> Vec<ScalarValue> {
    self
      .navigate_all(dot_path)
      .iter()
      .filter(|node| {
        !node
          .descendants()
          .any(|child| child.kind() == SyntaxKind::BLOCK_MAP || child.kind() == SyntaxKind::BLOCK_SEQ)
      })
      .filter_map(extract_scalar)
      .collect()
  }

  pub fn node_type(&self, dot_path: &str) -> NodeType {
    match self.navigate(dot_path) {
      Ok(node) => {
        if let Some(first_structural) = node
          .descendants()
          .find(|child| BlockMap::can_cast(child.kind()) || BlockSeq::can_cast(child.kind()))
        {
          if BlockMap::can_cast(first_structural.kind()) {
            NodeType::Map
          } else {
            NodeType::Sequence
          }
        } else if extract_scalar(&node).is_some() {
          NodeType::Scalar
        } else {
          NodeType::NotFound
        }
      }
      Err(_) => NodeType::NotFound,
    }
  }

  pub fn get_node_info(&self, dot_path: &str) -> NodeInfo {
    let selector = crate::selector::Selector::parse(dot_path);
    let source = self.root.text().to_string();

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
      None => NodeInfo {
        node_type: self.node_type(dot_path),
        is_list: false,
        value: None,
        list_values: vec![],
        location,
        key_name,
        key_location,
      },
    }
  }

  fn resolve_location(&self, dot_path: &str, source: &str) -> (Location, Option<String>, Location) {
    match self.navigate(dot_path) {
      Ok(node) => {
        let range = node.text_range();
        let location = compute_location(source, range.start().into(), range.end().into());

        let (key_name, key_location) = node
          .parent()
          .and_then(|parent| {
            use yaml_parser::ast::BlockMapEntry;

            BlockMapEntry::cast(parent).and_then(|entry| {
              entry.key().and_then(|key_node| {
                let key_text = extract_scalar_text(key_node.syntax())?;
                let key_range = key_node.syntax().text_range();
                let key_location = compute_location(source, key_range.start().into(), key_range.end().into());

                Some((key_text, key_location))
              })
            })
          })
          .map(|(name, location)| (Some(name), location))
          .unwrap_or((None, Location::default()));

        (location, key_name, key_location)
      }
      Err(_) => (Location::default(), None, Location::default()),
    }
  }

  pub fn find_items(&self, dot_path: &str, condition: Option<&str>, select: Option<&str>) -> Vec<serde_json::Value> {
    let values = match condition {
      Some(cond) => self.filter(dot_path, cond),
      None => self.get_values(dot_path),
    };

    let select_fields: Option<Vec<&str>> = select.map(|s| s.split(',').collect());

    values
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
      .collect()
  }

  pub fn get_value(&self, dot_path: &str) -> Option<serde_yaml::Value> {
    if dot_path.is_empty() {
      return Some(node_to_yaml_value(&self.root));
    }

    let nodes = self.navigate_all(dot_path);

    if nodes.is_empty() {
      return None;
    }

    if nodes.len() == 1 {
      return Some(node_to_yaml_value(&nodes[0]));
    }

    let values: Vec<serde_yaml::Value> = nodes.iter().map(node_to_yaml_value).collect();

    Some(serde_yaml::Value::Sequence(values))
  }

  pub fn get_values(&self, dot_path: &str) -> Vec<serde_yaml::Value> {
    self.navigate_all(dot_path).iter().map(node_to_yaml_value).collect()
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
    if dot_path.contains('[') {
      return !self.navigate_all(dot_path).is_empty();
    }

    self.get(dot_path).is_some()
  }

  pub fn get_sequence_values(&self, dot_path: &str) -> Vec<String> {
    let current_node = match self.navigate(dot_path) {
      Ok(node) => node,
      Err(_) => return Vec::new(),
    };

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Vec::new(),
    };

    sequence
      .entries()
      .filter_map(|entry| entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())))
      .collect()
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
