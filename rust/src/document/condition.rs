use super::*;

impl Document {
  pub fn filter(&self, dot_path: &str, condition: &str) -> Vec<yaml_serde::Value> {
    self
      .navigate_all_compact(dot_path)
      .iter()
      .filter(|node| self.evaluate_condition_on_node(node, condition))
      .map(node_to_yaml_value)
      .collect()
  }

  pub fn filter_with_selectors(&self, dot_path: &str, condition: &str) -> Vec<(yaml_serde::Value, String, usize)> {
    let source = self.root.text().to_string();

    self
      .navigate_all_compact(dot_path)
      .iter()
      .filter(|node| self.evaluate_condition_on_node(node, condition))
      .map(|node| {
        let offset: usize = node.text_range().start().into();
        let line = line_at(&source, offset);

        (node_to_yaml_value(node), super::get::node_selector(node), line)
      })
      .collect()
  }

  pub(super) fn evaluate_condition_on_node(&self, node: &SyntaxNode, condition: &str) -> bool {
    let condition = condition.trim();

    let (left, operator, right) = match parse_condition(condition) {
      Some(parts) => parts,
      None => return false,
    };

    let path = crate::selector::Selector::parse(&left);

    if !path.is_relative() {
      return false;
    }

    let target_nodes = navigate_from_node(node, &path.to_selector_string());
    let values: Vec<String> = target_nodes.iter().filter_map(extract_scalar_text).collect();

    match operator {
      "==" => values.iter().any(|value| value == &right),
      "!=" => values.iter().all(|value| value != &right),
      "contains" => {
        if values.iter().any(|value| value == &right || value.contains(&right)) {
          return true;
        }

        for node in &target_nodes {
          if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
            for entry in sequence.entries() {
              if let Some(text) = entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())) {
                if text == right {
                  return true;
                }
              }
            }
          }
        }

        false
      }
      "not_contains" => {
        for node in &target_nodes {
          if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
            for entry in sequence.entries() {
              if let Some(text) = entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())) {
                if text == right {
                  return false;
                }
              }
            }
          }
        }

        !values.iter().any(|value| value == &right || value.contains(&right))
      }
      _ => false,
    }
  }

  pub fn evaluate_condition(&self, parent_path: &str, condition: &str) -> bool {
    let condition = condition.trim();

    let (left, operator, right) = match parse_condition(condition) {
      Some(parts) => parts,
      None => return false,
    };

    let path = crate::selector::Selector::parse(&left);

    let full_path = if path.is_relative() {
      let path_string = path.to_selector_string();

      if parent_path.is_empty() {
        path_string
      } else {
        format!("{}.{}", parent_path, path_string)
      }
    } else {
      path.to_selector_string()
    };

    let has_brackets = crate::selector::Selector::parse(&full_path).has_brackets();

    match operator {
      "==" => {
        if has_brackets {
          self.get_all(&full_path).iter().any(|value| value == &right)
        } else {
          self.get(&full_path).unwrap_or_default() == right
        }
      }
      "!=" => {
        if has_brackets {
          self.get_all(&full_path).iter().all(|value| value != &right)
        } else {
          self.get(&full_path).unwrap_or_default() != right
        }
      }
      "contains" => {
        if has_brackets {
          self.get_all(&full_path).iter().any(|value| value == &right || value.contains(&right))
        } else {
          let items = self.get_sequence_values(&full_path);

          if !items.is_empty() {
            items.iter().any(|item| item == &right)
          } else {
            self.get(&full_path).map(|value| value.contains(&right)).unwrap_or(false)
          }
        }
      }
      "not_contains" => {
        if has_brackets {
          self.get_all(&full_path).iter().all(|value| value != &right && !value.contains(&right))
        } else {
          let items = self.get_sequence_values(&full_path);

          if !items.is_empty() {
            !items.iter().any(|item| item == &right)
          } else {
            self.get(&full_path).map(|value| !value.contains(&right)).unwrap_or(true)
          }
        }
      }
      _ => false,
    }
  }
}
