use std::process;
use std::sync::LazyLock;

use indoc::indoc;

use super::{colorize_examples, parse_file, resolve_files};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba get config.yml "database.host"
    yerba get videos.yml "[].title"
    yerba get videos.yml "[0].title"
    yerba get "data/**/videos.yml" "[].speakers[].name"
    yerba get videos.yml "[]" --select "title,speakers"
    yerba get videos.yml "[]" --condition ".kind == keynote"
    yerba get videos.yml "[]" --select "title" --condition ".kind == keynote"
    yerba get videos.yml "[]" --condition ".speakers contains Matz" --raw
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Get values, filter items, and select fields from YAML files",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
  /// Filter items by condition (e.g. '.kind == keynote', '.title contains Ruby')
  #[arg(long)]
  condition: Option<String>,
  /// Comma-separated fields to include (e.g. 'title,speakers')
  #[arg(long)]
  select: Option<String>,
  /// Output raw values (one per line) instead of JSON
  #[arg(long)]
  raw: bool,
  #[arg(long, hide = true)]
  json: bool,
}

impl Args {
  pub fn run(self) {
    let has_select = self.select.is_some();
    let has_condition = self.condition.is_some();
    let is_structured = has_select || has_condition;

    if is_structured {
      self.run_structured();
    } else {
      self.run_simple();
    }
  }

  fn run_simple(self) {
    let mut all_values: Vec<String> = Vec::new();

    for resolved_file in resolve_files(&self.file) {
      let document = parse_file(&resolved_file);

      let values = document.get_all(&self.selector);

      if values.is_empty() && !self.selector.contains('[') && !document.exists(&self.selector) {
        use super::color::*;
        eprintln!("{RED}Error:{RESET} path not found: {}", self.selector);
        process::exit(1);
      }

      all_values.extend(values);
    }

    if self.raw {
      for value in &all_values {
        println!("{}", value);
      }
    } else if all_values.len() == 1 {
      println!(
        "{}",
        serde_json::to_string_pretty(&all_values[0]).unwrap_or_else(|_| all_values[0].clone())
      );
    } else {
      println!(
        "{}",
        serde_json::to_string_pretty(&all_values).unwrap_or_else(|_| "[]".to_string())
      );
    }
  }

  fn run_structured(self) {
    let select_fields: Option<Vec<&str>> = self.select.as_deref().map(|fields| fields.split(',').collect());

    if self.raw {
      for resolved_file in resolve_files(&self.file) {
        let document = parse_file(&resolved_file);

        let matches = match &self.condition {
          Some(condition) => document.find_items(&self.selector, condition),
          None => document.find_all(&self.selector),
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
          Some(condition) => document.find_items(&self.selector, condition),
          None => document.find_all(&self.selector),
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
