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
    yerba get videos.yml "[]" --select ".title,.speakers"
    yerba get videos.yml "[]" --condition ".kind == keynote"
    yerba get videos.yml "[]" --select ".title" --condition ".kind == keynote"
    yerba get videos.yml "[].speakers[]" --condition "[].video_provider == youtube"
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
  /// Filter items by condition (e.g. '.kind == keynote', '[].video_provider == youtube')
  #[arg(long)]
  condition: Option<String>,
  /// Comma-separated fields to include (e.g. '.title,.speakers')
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
    let selector = yerba::Selector::parse(&self.selector);
    let condition_path = self.condition.as_deref().map(yerba::Selector::parse);
    let select_fields: Option<Vec<&str>> = self.select.as_deref().map(|fields| fields.split(',').collect());
    let (search_path, extract_field) = self.resolve_search_scope(&selector, condition_path.as_ref());

    let normalized_condition = self.condition.as_ref().map(|condition| {
      if !condition.starts_with('.') {
        if let Some(bracket_end) = condition.find(']') {
          let after_bracket = &condition[bracket_end + 1..];

          if after_bracket.starts_with('.') {
            return after_bracket.to_string();
          }
        }
      }

      condition.clone()
    });

    let search_path_string = search_path.to_selector_string();
    let mut all_results: Vec<serde_json::Value> = Vec::new();

    for resolved_file in resolve_files(&self.file) {
      let document = parse_file(&resolved_file);

      let values: Vec<serde_yaml::Value> = if let Some(condition) = &normalized_condition {
        document.filter(&search_path_string, condition)
      } else {
        document.get_values(&search_path_string)
      };

      if values.is_empty()
        && !selector.has_brackets()
        && normalized_condition.is_none()
        && !document.exists(&self.selector)
      {
        use super::color::*;
        eprintln!("{RED}Error:{RESET} path not found: {}", self.selector);
        process::exit(1);
      }

      for value in values {
        if let Some(field) = &extract_field {
          let field_string = field.to_selector_string();
          let json_value = yerba::json::resolve_select_field(&value, &field_string);

          if field.ends_with_bracket() {
            if let serde_json::Value::Array(items) = json_value {
              all_results.extend(items);
            }
          } else {
            all_results.push(json_value);
          }

          continue;
        }

        if let Some(fields) = &select_fields {
          let mut result = serde_json::Map::new();

          result.insert("__file".to_string(), serde_json::Value::String(resolved_file.clone()));

          for field in fields {
            let json_value = yerba::json::resolve_select_field(&value, field);
            let json_key = yerba::json::select_field_key(field);

            result.insert(json_key, json_value);
          }

          all_results.push(serde_json::Value::Object(result));

          continue;
        }

        all_results.push(yerba::json::yaml_to_json(&value));
      }
    }

    if self.raw {
      for value in &all_results {
        match value {
          serde_json::Value::String(string) => println!("{}", string),
          serde_json::Value::Null => println!("null"),
          serde_json::Value::Bool(boolean) => println!("{}", boolean),
          serde_json::Value::Number(number) => println!("{}", number),
          _ => println!("{}", serde_json::to_string(value).unwrap_or_default()),
        }
      }
    } else if all_results.len() == 1 {
      println!(
        "{}",
        serde_json::to_string_pretty(&all_results[0]).unwrap_or_else(|_| "null".to_string())
      );
    } else {
      println!(
        "{}",
        serde_json::to_string_pretty(&all_results).unwrap_or_else(|_| "[]".to_string())
      );
    }
  }

  fn resolve_search_scope(
    &self,
    selector: &yerba::Selector,
    condition_path: Option<&yerba::Selector>,
  ) -> (yerba::Selector, Option<yerba::Selector>) {
    if let Some(condition) = condition_path {
      if condition.is_relative() {
        let (container, field) = selector.split_at_last_bracket();

        if container.is_empty() {
          return (selector.clone(), None);
        }

        let extract = if field.is_empty() { None } else { Some(field) };

        return (container, extract);
      } else {
        let (container, _) = condition.split_at_first_bracket();
        let container_string = container.to_selector_string();
        let selector_string = selector.to_selector_string();

        if selector_string.starts_with(&container_string) {
          let rest = selector_string[container_string.len()..].trim_start_matches('.');

          if rest.is_empty() {
            return (container, None);
          }

          return (container, Some(yerba::Selector::parse(&format!(".{}", rest))));
        }

        return (container, None);
      }
    }

    if selector.has_brackets() && !selector.ends_with_bracket() {
      let (container, field) = selector.split_at_last_bracket();

      if !container.is_empty() {
        let extract = if field.is_empty() { None } else { Some(field) };

        return (container, extract);
      }
    }

    (selector.clone(), None)
  }
}
