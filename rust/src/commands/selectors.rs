use std::collections::BTreeMap;
use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{color, parse_file, resolve_files};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba selectors config.yml
    yerba selectors config.yml "database"
    yerba selectors videos.yml "[]"
    yerba selectors videos.yml "[].speakers"
    yerba selectors "data/**/videos.yml"
    yerba selectors "data/**/videos.yml" "[].talks"
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Show all valid selectors in a YAML file",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  /// Show selectors starting from this path
  selector: Option<String>,
  /// Sort selectors alphabetically (default: document order)
  #[arg(long)]
  sorted: bool,
}

#[derive(Debug, Default)]
struct SelectorInfo {
  min_count: Option<usize>,
  max_count: Option<usize>,
  order: usize,
}

impl SelectorInfo {
  fn record_count(&mut self, count: usize) {
    self.min_count = Some(self.min_count.map_or(count, |min| min.min(count)));
    self.max_count = Some(self.max_count.map_or(count, |max| max.max(count)));
  }

  fn count_label(&self) -> Option<String> {
    match (self.min_count, self.max_count) {
      (Some(0), Some(0)) => Some("empty".to_string()),

      (Some(min), Some(max)) if min == max => {
        if max <= 5 {
          let indices: Vec<String> = (0..max).map(|i| format!("[{}]", i)).collect();

          Some(indices.join(", "))
        } else {
          Some(format!("[0]..[{}]", max - 1))
        }
      }

      (Some(min), Some(max)) => {
        if max <= 5 {
          let indices: Vec<String> = (0..max).map(|i| format!("[{}]", i)).collect();

          Some(format!("{} ({}-{} items)", indices.join(", "), min, max))
        } else {
          Some(format!("[0]..[{}] ({}-{} items)", max - 1, min, max))
        }
      }
      _ => None,
    }
  }
}

impl Args {
  pub fn run(self) {
    let mut all_selectors: BTreeMap<String, SelectorInfo> = BTreeMap::new();
    let mut counter: usize = 1;
    let selector = self.selector.as_deref().unwrap_or("");

    for resolved_file in resolve_files(&self.file) {
      let document = parse_file(&resolved_file);
      let prefix = if selector.is_empty() { String::new() } else { selector.to_string() };

      let values = if selector.is_empty() {
        document.get_value("").into_iter().collect::<Vec<_>>()
      } else {
        document.get_values(selector)
      };

      for value in values {
        collect_selectors(&value, &prefix, &mut all_selectors, &mut counter);
      }
    }

    let mut entries: Vec<(&String, &SelectorInfo)> = all_selectors.iter().collect();

    if self.sorted {
      entries.sort_by_key(|(name, _)| name.to_string());
    } else {
      entries.sort_by_key(|(_, info)| info.order);
    }

    let max_selector_len = entries.iter().map(|(selector, _)| selector.len()).max().unwrap_or(0);

    for (selector, info) in &entries {
      if let Some(label) = info.count_label() {
        let padding = max_selector_len - selector.len() + 2;

        println!("{}{}{}{}{}", selector, " ".repeat(padding), color::DIM, label, color::RESET);
      } else {
        println!("{}", selector);
      }
    }
  }
}

fn collect_selectors(value: &yaml_serde::Value, prefix: &str, selectors: &mut BTreeMap<String, SelectorInfo>, counter: &mut usize) {
  match value {
    yaml_serde::Value::Mapping(map) => {
      for (key, child) in map {
        if let yaml_serde::Value::String(key_string) = key {
          let selector = if prefix.is_empty() {
            key_string.clone()
          } else {
            format!("{}.{}", prefix, key_string)
          };

          let entry = selectors.entry(selector.clone()).or_default();
          if entry.order == 0 {
            entry.order = *counter;
            *counter += 1;
          }

          collect_selectors(child, &selector, selectors, counter);
        }
      }
    }

    yaml_serde::Value::Sequence(sequence) => {
      let bracket_prefix = format!("{}[]", prefix);

      let entry = selectors.entry(bracket_prefix.clone()).or_default();
      entry.record_count(sequence.len());
      if entry.order == 0 {
        entry.order = *counter;
        *counter += 1;
      }

      for item in sequence {
        collect_selectors(item, &bracket_prefix, selectors, counter);
      }
    }

    _ => {}
  }
}
