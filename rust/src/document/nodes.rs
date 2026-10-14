use super::*;

#[derive(Debug)]
pub struct NodeRecord {
  pub parent: Option<usize>,
  pub node_type: NodeType,
  pub selector: String,
  pub key_name: Option<String>,
  pub scalar: Option<ScalarValue>,
  pub has_value: bool,
  pub location: Location,
  pub key_location: Location,
}

fn location_of(lines: &LineIndex, range: TextRange) -> Location {
  compute_location(lines, range.start().into(), range.end().into())
}

fn collection_type(node: &SyntaxNode) -> Option<NodeType> {
  let mut flow = None;

  for descendant in node.descendants() {
    match descendant.kind() {
      SyntaxKind::BLOCK_SEQ => return Some(NodeType::Sequence),
      SyntaxKind::BLOCK_MAP => return Some(NodeType::Map),
      SyntaxKind::FLOW_SEQ if flow.is_none() => flow = Some(NodeType::Sequence),
      SyntaxKind::FLOW_MAP if flow.is_none() => flow = Some(NodeType::Map),
      _ => {}
    }
  }

  flow
}

struct Walker<'a> {
  lines: &'a LineIndex,
  records: Vec<NodeRecord>,
}

impl Walker<'_> {
  fn push(&mut self, parent: Option<usize>, node: Option<&SyntaxNode>, selector: String, key: Option<(String, Location)>) {
    let collection = node.and_then(collection_type);
    let scalar = match (node, collection) {
      (Some(node), None) => extract_scalar(node),
      _ => None,
    };
    let node_type = match (collection, &scalar) {
      (Some(node_type), _) => node_type,
      (None, Some(_)) => NodeType::Scalar,
      (None, None) => NodeType::NotFound,
    };
    let location = match node {
      Some(node) if parent.is_some() || !matches!(node_type, NodeType::Sequence) => location_of(self.lines, node.text_range()),
      _ => Location::default(),
    };
    let (key_name, key_location) = key.map(|(name, location)| (Some(name), location)).unwrap_or((None, Location::default()));

    let index = self.records.len();

    self.records.push(NodeRecord {
      parent,
      node_type,
      selector: selector.clone(),
      key_name,
      scalar,
      has_value: true,
      location,
      key_location,
    });

    if let Some(node) = node {
      match node_type {
        NodeType::Sequence => self.sequence_children(index, node, &selector),
        NodeType::Map => self.map_children(index, node, &selector),
        _ => {}
      }
    }
  }

  fn sequence_children(&mut self, index: usize, node: &SyntaxNode, selector: &str) {
    let entries: Vec<SyntaxNode> = if let Some(sequence) = find_block_sequence(node) {
      sequence.entries().map(|entry| entry.syntax().clone()).collect()
    } else if let Some(sequence) = find_flow_sequence(node) {
      flow_sequence_entries(&sequence)
    } else {
      Vec::new()
    };

    for (position, entry) in entries.iter().enumerate() {
      self.push(Some(index), Some(entry), format!("{}[{}]", selector, position), None);
    }
  }

  fn map_children(&mut self, index: usize, node: &SyntaxNode, selector: &str) {
    let entries: Vec<(SyntaxNode, Option<SyntaxNode>, Option<SyntaxNode>)> = if let Some(map) = find_block_map(node) {
      map
        .entries()
        .map(|entry| {
          (
            entry.syntax().clone(),
            entry.key().map(|key| key.syntax().clone()),
            entry.value().map(|value| value.syntax().clone()),
          )
        })
        .collect()
    } else if let Some(map) = find_flow_map(node) {
      flow_map_entries(&map)
        .into_iter()
        .map(|entry| {
          (
            entry.syntax().clone(),
            entry.key().map(|key| key.syntax().clone()),
            entry.value().map(|value| value.syntax().clone()),
          )
        })
        .collect()
    } else {
      Vec::new()
    };

    for (entry_node, key_node, value_node) in entries {
      let Some(key_node) = key_node else { continue };
      let Some(key_name) = extract_scalar_text(&key_node) else { continue };

      let key_location = location_of(self.lines, key_node.text_range());
      let child_selector = if selector.is_empty() {
        key_name.clone()
      } else {
        format!("{}.{}", selector, key_name)
      };

      match value_node {
        Some(value_node) => self.push(Some(index), Some(&value_node), child_selector, Some((key_name, key_location))),
        None => self.records.push(NodeRecord {
          parent: Some(index),
          node_type: NodeType::Scalar,
          selector: child_selector,
          key_name: Some(key_name),
          scalar: None,
          has_value: false,
          location: location_of(self.lines, entry_node.text_range()),
          key_location,
        }),
      }
    }
  }
}

impl Document {
  pub fn node_records(&self) -> Vec<NodeRecord> {
    let mut walker = Walker {
      lines: self.line_index(),
      records: Vec::new(),
    };

    if let Some(document) = Root::cast(self.root.clone()).and_then(|root| root.documents().next()) {
      walker.push(None, Some(document.syntax()), String::new(), None);
    }

    walker.records
  }
}
