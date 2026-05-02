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

    _ => {
      let prefix = " ".repeat(indent);

      format!("{}{}", prefix, format_yaml_scalar(value, quote_style))
    }
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
