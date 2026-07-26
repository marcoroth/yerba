use super::*;

const CONDITION_OPERATORS: &str = "expected `<selector> <operator> <value>`, where operator is one of ==, !=, contains, not_contains";

pub fn validate_condition(condition: &str) -> Result<(), YerbaError> {
  let trimmed = condition.trim();

  if trimmed.is_empty() {
    return Err(YerbaError::InvalidCondition(condition.to_string(), "condition is empty".to_string()));
  }

  let Some((left, _operator, _right)) = parse_condition(trimmed) else {
    return Err(YerbaError::InvalidCondition(condition.to_string(), CONDITION_OPERATORS.to_string()));
  };

  if crate::selector::Selector::parse(&left).is_empty() {
    return Err(YerbaError::InvalidCondition(
      condition.to_string(),
      "condition is missing a selector on the left of the operator".to_string(),
    ));
  }

  Ok(())
}

pub fn validate_item_condition(condition: &str) -> Result<(), YerbaError> {
  validate_condition(condition)?;

  let trimmed = condition.trim();

  let Some((left, _operator, _right)) = parse_condition(trimmed) else {
    return Ok(());
  };

  if !crate::selector::Selector::parse(&left).is_relative() {
    return Err(YerbaError::InvalidCondition(
      condition.to_string(),
      format!(
        "selector \"{}\" is absolute, but this condition is tested against each item, so it must be relative and start with `.` — did you mean \".{}\"?",
        left.trim(),
        left.trim()
      ),
    ));
  }

  Ok(())
}

impl Document {
  pub fn filter(&self, dot_path: &str, condition: &str) -> Result<Vec<yaml_serde::Value>, YerbaError> {
    validate_item_condition(condition)?;

    Ok(
      self
        .navigate_all_compact(dot_path)
        .iter()
        .filter(|node| self.evaluate_condition_on_node(node, condition))
        .map(node_to_yaml_value)
        .collect(),
    )
  }

  pub fn filter_with_selectors(&self, dot_path: &str, condition: &str) -> Result<Vec<(yaml_serde::Value, String, usize)>, YerbaError> {
    validate_item_condition(condition)?;

    let source = self.source_text();

    Ok(
      self
        .navigate_all_compact(dot_path)
        .iter()
        .filter(|node| self.evaluate_condition_on_node(node, condition))
        .map(|node| {
          let offset: usize = node.text_range().start().into();
          let line = line_at(&source, offset);

          (node_to_yaml_value(node), super::get::node_selector(node), line)
        })
        .collect(),
    )
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
          if let Some(sequence) = find_block_sequence(node) {
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
          if let Some(sequence) = find_block_sequence(node) {
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

  pub fn evaluate_condition(&self, parent_path: &str, condition: &str) -> Result<bool, YerbaError> {
    if parent_path.is_empty() {
      validate_condition(condition)?;
    } else {
      validate_item_condition(condition)?;
    }

    let condition = condition.trim();

    let (left, operator, right) = match parse_condition(condition) {
      Some(parts) => parts,
      None => return Ok(false),
    };

    let path = crate::selector::Selector::parse(&left);
    let path_string = path.to_selector_string();

    let full_path = if parent_path.is_empty() {
      path_string
    } else {
      format!("{}.{}", parent_path, path_string)
    };

    let has_brackets = crate::selector::Selector::parse(&full_path).has_brackets();

    let result = match operator {
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
    };

    Ok(result)
  }
}
