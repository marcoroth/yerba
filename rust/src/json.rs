use crate::selector::{Selector, SelectorSegment};

/// Serializes `value` as JSON, writing non-finite floats as `NaN`, `Infinity` and `-Infinity`
/// (which Ruby's `JSON.parse` reads with `allow_nan: true`) instead of dropping them to `null`.
pub fn yaml_to_json_string_allowing_nan(value: &yaml_serde::Value) -> String {
  let mut output = String::new();
  write_json_allowing_nan(value, &mut output);
  output
}

fn write_json_allowing_nan(value: &yaml_serde::Value, output: &mut String) {
  match value {
    yaml_serde::Value::Number(number) if number.is_nan() => output.push_str("NaN"),
    yaml_serde::Value::Number(number) if number.is_infinite() => output.push_str(if number.as_f64().is_some_and(f64::is_sign_negative) {
      "-Infinity"
    } else {
      "Infinity"
    }),

    yaml_serde::Value::Sequence(sequence) => {
      output.push('[');

      for (index, item) in sequence.iter().enumerate() {
        if index > 0 {
          output.push(',');
        }

        write_json_allowing_nan(item, output);
      }

      output.push(']');
    }

    yaml_serde::Value::Mapping(mapping) => {
      output.push('{');

      for (index, (key, item)) in mapping.iter().enumerate() {
        if index > 0 {
          output.push(',');
        }

        let key = match key {
          yaml_serde::Value::String(string) => string.clone(),
          _ => format!("{:?}", key),
        };

        output.push_str(&serde_json::Value::String(key).to_string());
        output.push(':');
        write_json_allowing_nan(item, output);
      }

      output.push('}');
    }

    yaml_serde::Value::Tagged(tagged) => write_json_allowing_nan(&tagged.value, output),

    _ => output.push_str(&yaml_to_json(value).to_string()),
  }
}

pub fn yaml_to_json(value: &yaml_serde::Value) -> serde_json::Value {
  match value {
    yaml_serde::Value::Null => serde_json::Value::Null,
    yaml_serde::Value::Bool(boolean) => serde_json::Value::Bool(*boolean),

    yaml_serde::Value::Number(number) => {
      if let Some(integer) = number.as_i64() {
        serde_json::Value::Number(integer.into())
      } else if let Some(integer) = number.as_u64() {
        serde_json::Value::Number(integer.into())
      } else if let Some(float) = number.as_f64() {
        serde_json::json!(float)
      } else {
        serde_json::Value::String(number.to_string())
      }
    }

    yaml_serde::Value::String(string) => serde_json::Value::String(string.clone()),

    yaml_serde::Value::Sequence(sequence) => serde_json::Value::Array(sequence.iter().map(yaml_to_json).collect()),

    yaml_serde::Value::Mapping(mapping) => {
      let mut map = serde_json::Map::new();

      for (key, yaml_value) in mapping {
        let json_key = match key {
          yaml_serde::Value::String(string) => string.clone(),
          _ => format!("{:?}", key),
        };

        map.insert(json_key, yaml_to_json(yaml_value));
      }

      serde_json::Value::Object(map)
    }

    yaml_serde::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
  }
}

pub fn resolve_select_field(value: &yaml_serde::Value, field: &str) -> serde_json::Value {
  let parsed = Selector::parse(field);
  let segments = parsed.segments();

  if segments.len() == 1 {
    if let SelectorSegment::Key(key) = &segments[0] {
      if let yaml_serde::Value::Mapping(map) = value {
        for (map_key, yaml_value) in map {
          if let yaml_serde::Value::String(key_string) = map_key {
            if key_string == key {
              return yaml_to_json(yaml_value);
            }
          }
        }
      }

      return serde_json::Value::Null;
    }
  }

  let mut current_values = vec![value.clone()];

  for segment in segments {
    let mut next_values = Vec::new();

    for current in &current_values {
      match segment {
        SelectorSegment::AllItems => {
          if let yaml_serde::Value::Sequence(sequence) = current {
            next_values.extend(sequence.iter().cloned());
          }
        }

        SelectorSegment::AllKeys => {
          if let yaml_serde::Value::Mapping(mapping) = current {
            next_values.extend(mapping.iter().map(|(_, entry)| entry.clone()));
          }
        }

        SelectorSegment::Index(_) | SelectorSegment::IndexFromEnd(_) => {
          if let yaml_serde::Value::Sequence(sequence) = current {
            if let Some(item) = segment.sequence_index(sequence.len()).and_then(|index| sequence.get(index)) {
              next_values.push(item.clone());
            }
          }
        }

        SelectorSegment::Key(key) => {
          if let yaml_serde::Value::Mapping(map) = current {
            for (map_key, yaml_value) in map {
              if let yaml_serde::Value::String(key_string) = map_key {
                if key_string == key {
                  next_values.push(yaml_value.clone());
                }
              }
            }
          }
        }
      }
    }

    current_values = next_values;
  }

  let used_all_items = parsed
    .segments()
    .iter()
    .any(|s| matches!(s, SelectorSegment::AllItems | SelectorSegment::AllKeys));

  if current_values.is_empty() {
    if used_all_items {
      serde_json::Value::Array(Vec::new())
    } else {
      serde_json::Value::Null
    }
  } else if current_values.len() == 1 && !used_all_items {
    yaml_to_json(&current_values[0])
  } else {
    serde_json::Value::Array(current_values.iter().map(yaml_to_json).collect())
  }
}

pub fn select_field_key(field: &str) -> String {
  let parsed = Selector::parse(field);

  parsed
    .segments()
    .iter()
    .find_map(|segment| if let SelectorSegment::Key(key) = segment { Some(key.clone()) } else { None })
    .unwrap_or_else(|| field.to_string())
}
