use serde_json::Value;

use crate::QuoteStyle;

pub fn json_to_yaml_text(value: &Value, quote_style: &QuoteStyle, indent: usize) -> String {
  match value {
    Value::Object(map) => {
      let prefix = " ".repeat(indent);

      map
        .iter()
        .map(|(k, v)| match v {
          Value::Array(arr) => {
            let items: Vec<String> = arr
              .iter()
              .map(|item| match item {
                Value::Object(_) => {
                  let inner = json_to_yaml_text(item, quote_style, indent + 4);
                  format!("{}  - {}", prefix, inner.trim_start())
                }

                _ => format!("{}  - {}", prefix, format_yaml_scalar(item, quote_style)),
              })
              .collect();

            format!("{}{}:\n{}", prefix, k, items.join("\n"))
          }

          Value::Object(_) => {
            let inner = json_to_yaml_text(v, quote_style, indent + 2);
            format!("{}{}:\n{}", prefix, k, inner)
          }

          _ => format!("{}{}: {}", prefix, k, format_yaml_scalar(v, quote_style)),
        })
        .collect::<Vec<_>>()
        .join("\n")
    }

    Value::Array(array) => {
      let prefix = " ".repeat(indent);

      array
        .iter()
        .map(|item| match item {
          Value::Object(_) => {
            let inner = json_to_yaml_text(item, quote_style, indent + 2);
            format!("{}- {}", prefix, inner.trim_start())
          }

          _ => format!("{}- {}", prefix, format_yaml_scalar(item, quote_style)),
        })
        .collect::<Vec<_>>()
        .join("\n")
    }

    _ => {
      let prefix = " ".repeat(indent);

      format!("{}{}", prefix, format_yaml_scalar(value, quote_style))
    }
  }
}

pub fn yaml_value_to_flow_text(value: &yaml_serde::Value) -> String {
  match value {
    yaml_serde::Value::Null => "null".to_string(),
    yaml_serde::Value::Bool(boolean) => boolean.to_string(),
    yaml_serde::Value::Number(number) => number.to_string(),

    yaml_serde::Value::String(string) => {
      if crate::syntax::is_yaml_non_string(string) {
        format!("\"{}\"", string.replace('"', "\\\""))
      } else {
        string.clone()
      }
    }

    yaml_serde::Value::Sequence(sequence) => {
      let items: Vec<String> = sequence.iter().map(yaml_value_to_flow_text).collect();

      format!("[{}]", items.join(", "))
    }

    yaml_serde::Value::Mapping(mapping) => {
      let pairs: Vec<String> = mapping
        .iter()
        .map(|(key, value)| {
          let key_string = match key {
            yaml_serde::Value::String(string) => string.clone(),
            _ => format!("{:?}", key),
          };

          format!("{}: {}", key_string, yaml_value_to_flow_text(value))
        })
        .collect();

      format!("{{{}}}", pairs.join(", "))
    }

    yaml_serde::Value::Tagged(tagged) => yaml_value_to_flow_text(&tagged.value),
  }
}

pub fn yaml_value_to_block_text(value: &yaml_serde::Value, indent: usize) -> String {
  let prefix = " ".repeat(indent);

  match value {
    yaml_serde::Value::Sequence(sequence) => sequence
      .iter()
      .map(|item| match item {
        yaml_serde::Value::Mapping(_) => {
          let inner = yaml_value_to_block_text(item, indent + 2);
          format!("{}- {}", prefix, inner.trim_start())
        }

        _ => format!("{}- {}", prefix, yaml_value_to_flow_text(item)),
      })
      .collect::<Vec<_>>()
      .join("\n"),

    yaml_serde::Value::Mapping(mapping) => mapping
      .iter()
      .map(|(key, value)| {
        let key_string = match key {
          yaml_serde::Value::String(string) => string.clone(),
          _ => format!("{:?}", key),
        };
        match value {
          yaml_serde::Value::Sequence(_) | yaml_serde::Value::Mapping(_) => {
            let inner = yaml_value_to_block_text(value, indent + 2);

            format!("{}{}:\n{}", prefix, key_string, inner)
          }

          _ => format!("{}{}: {}", prefix, key_string, yaml_value_to_flow_text(value)),
        }
      })
      .collect::<Vec<_>>()
      .join("\n"),

    _ => format!("{}{}", prefix, yaml_value_to_flow_text(value)),
  }
}

fn format_yaml_scalar(value: &Value, quote_style: &QuoteStyle) -> String {
  match value {
    Value::Null => "null".to_string(),
    Value::Bool(boolean) => boolean.to_string(),
    Value::Number(number) => number.to_string(),
    Value::String(string) => match quote_style {
      QuoteStyle::Double => {
        let escaped = string.replace('\\', "\\\\").replace('"', "\\\"");

        format!("\"{}\"", escaped)
      }

      QuoteStyle::Single => {
        let escaped = string.replace('\'', "''");

        format!("'{}'", escaped)
      }

      QuoteStyle::Plain => {
        if crate::syntax::is_yaml_non_string(string) {
          format!("\"{}\"", string.replace('"', "\\\""))
        } else {
          string.clone()
        }
      }

      _ => {
        let escaped = string.replace('\\', "\\\\").replace('"', "\\\"");

        format!("\"{}\"", escaped)
      }
    },

    Value::Array(array) => {
      let items: Vec<String> = array.iter().map(|item| format_yaml_scalar(item, quote_style)).collect();

      format!("[{}]", items.join(", "))
    }

    Value::Object(_) => format!("{:?}", value),
  }
}
