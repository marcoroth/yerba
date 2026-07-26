use crate::selector::{Selector, SelectorSegment};

pub fn yaml_to_json(value: &yaml_serde::Value) -> serde_json::Value {
  match value {
    yaml_serde::Value::Null => serde_json::Value::Null,
    yaml_serde::Value::Bool(boolean) => serde_json::Value::Bool(*boolean),

    yaml_serde::Value::Number(number) => {
      if let Some(integer) = number.as_i64() {
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

        SelectorSegment::Index(index) => {
          if let yaml_serde::Value::Sequence(sequence) = current {
            if let Some(item) = sequence.get(*index) {
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
