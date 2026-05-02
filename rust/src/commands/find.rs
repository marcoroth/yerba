use indoc::indoc;

use super::{parse_file, resolve_files};

#[derive(clap::Args)]
#[command(
  about = "Find and filter items with conditions, output as JSON or raw YAML",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba find "data/**/videos.yml" "[]" --condition '.kind == keynote'
      yerba find "data/**/videos.yml" "[]" --select 'id,title' --condition '.kind == keynote'
      yerba find "data/**/videos.yml" "[]" --select 'id,title,speakers[0].name'
      yerba find "data/**/videos.yml" "[]" --condition '.title contains Ruby' --raw
      yerba find "data/**/videos.yml" "[]" --condition '.speakers contains "Matz"'
  "#}
)]
pub struct Args {
  file: String,
  path: String,
  #[arg(long)]
  condition: Option<String>,
  /// Comma-separated fields to include (supports dot paths like speakers[].name)
  #[arg(long)]
  select: Option<String>,
  /// Output raw YAML instead of JSON
  #[arg(long)]
  raw: bool,
}

impl Args {
  pub fn run(self) {
    let select_fields: Option<Vec<&str>> = self.select.as_deref().map(|fields| fields.split(',').collect());

    if self.raw {
      for resolved_file in resolve_files(&self.file) {
        let document = parse_file(&resolved_file);

        let matches = match &self.condition {
          Some(condition) => document.find_items(&self.path, condition),
          None => document.find_all(&self.path),
        };

        for (index, item) in matches.iter().enumerate() {
          if index > 0 {
            println!();
          }

          eprintln!("# {}:{}", resolved_file, item.line);
          println!("{}", item.text);
        }
      }
    } else {
      let mut all_results: Vec<serde_json::Value> = Vec::new();

      for resolved_file in resolve_files(&self.file) {
        let document = parse_file(&resolved_file);

        let matches = match &self.condition {
          Some(condition) => document.find_items(&self.path, condition),
          None => document.find_all(&self.path),
        };

        for item in &matches {
          let yaml_with_dash = format!("- {}", item.text.trim_start_matches("- "));

          if let Ok(parsed) = serde_yaml::from_str::<Vec<serde_yaml::Value>>(&yaml_with_dash) {
            for value in parsed {
              let mut result = serde_json::Map::new();

              result.insert("__file".to_string(), serde_json::Value::String(resolved_file.clone()));

              result.insert("__line".to_string(), serde_json::Value::Number(item.line.into()));

              match &select_fields {
                Some(fields) => {
                  for field in fields {
                    let json_value = yerba::json::resolve_select_field(&value, field);
                    let json_key = yerba::json::select_field_key(field);

                    result.insert(json_key, json_value);
                  }
                }

                None => {
                  if let serde_yaml::Value::Mapping(map) = &value {
                    for (key, yaml_value) in map {
                      let json_key = match key {
                        serde_yaml::Value::String(string) => string.clone(),
                        _ => format!("{:?}", key),
                      };

                      result.insert(json_key, yerba::json::yaml_to_json(yaml_value));
                    }
                  }
                }
              }

              all_results.push(serde_json::Value::Object(result));
            }
          }
        }
      }

      println!(
        "{}",
        serde_json::to_string_pretty(&all_results).unwrap_or_else(|_| "[]".to_string())
      );
    }
  }
}
