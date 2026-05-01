pub fn yaml_to_json(value: &serde_yaml::Value) -> serde_json::Value {
  match value {
    serde_yaml::Value::Null => serde_json::Value::Null,
    serde_yaml::Value::Bool(boolean) => serde_json::Value::Bool(*boolean),

    serde_yaml::Value::Number(number) => {
      if let Some(integer) = number.as_i64() {
        serde_json::Value::Number(integer.into())
      } else if let Some(float) = number.as_f64() {
        serde_json::json!(float)
      } else {
        serde_json::Value::String(number.to_string())
      }
    }

    serde_yaml::Value::String(string) => serde_json::Value::String(string.clone()),

    serde_yaml::Value::Sequence(sequence) => {
      serde_json::Value::Array(sequence.iter().map(yaml_to_json).collect())
    }

    serde_yaml::Value::Mapping(mapping) => {
      let mut map = serde_json::Map::new();

      for (key, yaml_value) in mapping {
        let json_key = match key {
          serde_yaml::Value::String(string) => string.clone(),
          _ => format!("{:?}", key),
        };

        map.insert(json_key, yaml_to_json(yaml_value));
      }

      serde_json::Value::Object(map)
    }

    serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
  }
}

pub fn resolve_select_field(value: &serde_yaml::Value, field: &str) -> serde_json::Value {
  if !field.contains('.') && !field.contains('[') {
    if let serde_yaml::Value::Mapping(map) = value {
      for (key, yaml_value) in map {
        if let serde_yaml::Value::String(key_string) = key {
          if key_string == field {
            return yaml_to_json(yaml_value);
          }
        }
      }
    }

    return serde_json::Value::Null;
  }

  let mut current_values = vec![value.clone()];

  for segment in parse_select_segments(field) {
    let mut next_values = Vec::new();

    for current in &current_values {
      if segment.starts_with('[') {
        if let serde_yaml::Value::Sequence(sequence) = current {
          let index = segment
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
            .and_then(|inner| {
              if inner.is_empty() {
                None
              } else {
                inner.parse::<usize>().ok()
              }
            });

          match index {
            None => next_values.extend(sequence.iter().cloned()),
            Some(index) => {
              if let Some(item) = sequence.get(index) {
                next_values.push(item.clone());
              }
            }
          }
        }
      } else if let serde_yaml::Value::Mapping(map) = current {
        for (map_key, yaml_value) in map {
          if let serde_yaml::Value::String(key_string) = map_key {
            if key_string == segment {
              next_values.push(yaml_value.clone());
            }
          }
        }
      }
    }

    current_values = next_values;
  }

  let used_brackets = field.contains("[]");

  if current_values.is_empty() {
    if used_brackets {
      serde_json::Value::Array(Vec::new())
    } else {
      serde_json::Value::Null
    }
  } else if current_values.len() == 1 && !used_brackets {
    yaml_to_json(&current_values[0])
  } else {
    serde_json::Value::Array(current_values.iter().map(yaml_to_json).collect())
  }
}

pub fn select_field_key(field: &str) -> String {
  let root = field.split(['.', '[']).next().unwrap_or(field);

  root.to_string()
}

fn parse_select_segments(field: &str) -> Vec<&str> {
  let mut segments = Vec::new();
  let mut rest = field;

  while !rest.is_empty() {
    if rest.starts_with('[') {
      if let Some(close) = rest.find(']') {
        segments.push(&rest[..close + 1]);
        rest = &rest[close + 1..];

        if rest.starts_with('.') {
          rest = &rest[1..];
        }
      } else {
        segments.push(rest);
        break;
      }
    } else {
      let dot_index = rest.find('.');
      let bracket_index = rest.find('[');

      let split_at = match (dot_index, bracket_index) {
        (Some(dot), Some(bracket)) => Some(dot.min(bracket)),
        (Some(dot), None) => Some(dot),
        (None, Some(bracket)) => Some(bracket),
        (None, None) => None,
      };

      match split_at {
        Some(index) => {
          let segment = &rest[..index];

          if !segment.is_empty() {
            segments.push(segment);
          }

          rest = &rest[index..];

          if rest.starts_with('.') {
            rest = &rest[1..];
          }
        }

        None => {
          segments.push(rest);
          break;
        }
      }
    }
  }

  segments
}
