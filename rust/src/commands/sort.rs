use std::process;
use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba sort config.yml "tags"
    yerba sort videos.yml --by ".title"
    yerba sort videos.yml --by ".date" --order desc --by ".title"
    yerba sort videos.yml "[].speakers" --by ".name"
    yerba sort videos.yml "[]" --by ".id" --order "talk-3,talk-1,talk-2"
    yerba sort speakers.yml "[]" --by ".name" --order "Charlie,Alice,Bob"
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Sort or reorder items in a sequence",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  /// Selector (optional — omit for root-level sequence)
  selector: Option<String>,
  /// Field to sort or match by (can be repeated for tie-breakers)
  #[arg(long, action = clap::ArgAction::Append)]
  by: Vec<String>,
  /// Sort direction (asc/desc) or explicit order (comma-separated values, must list all items)
  #[arg(long, action = clap::ArgAction::Append)]
  order: Vec<String>,
  /// Additional fields to display alongside values (e.g. --context ".title")
  #[arg(long, action = clap::ArgAction::Append)]
  context: Vec<String>,
  /// Case-sensitive sort (default: case-insensitive)
  #[arg(long)]
  case_sensitive: bool,
  #[arg(long)]
  dry_run: bool,
}

fn is_direction(value: &str) -> bool {
  matches!(value, "asc" | "desc" | "ascending" | "descending")
}

fn is_explicit_reorder(orders: &[String]) -> bool {
  orders.iter().any(|o| !is_direction(o))
}

impl Args {
  pub fn run(self) {
    use super::color::*;

    if self.by.is_empty() && self.order.is_empty() && self.selector.is_none() {
      let document = parse_file(&self.file);

      eprintln!("{RED}Error:{RESET} specify a selector, --by, or --order");
      eprintln!();
      eprintln!("  {BOLD}Examples:{RESET}");
      eprintln!("    yerba sort \"{}\" \"tags\"", self.file);
      eprintln!("    yerba sort \"{}\" --by \".title\" --order asc", self.file);
      eprintln!();

      super::show_similar_selectors(&self.file, &document, "[]");

      process::exit(1);
    }

    if !self.by.is_empty() && self.order.is_empty() {
      self.show_values();
    } else if is_explicit_reorder(&self.order) {
      self.run_reorder();
    } else {
      self.run_sort();
    }
  }

  fn show_values(self) {
    use super::color::*;

    if self.by.len() != 1 {
      eprintln!("{RED}Error:{RESET} --order is required when using --by");
      process::exit(1);
    }

    let by = &self.by[0];
    let selector = self.selector.as_deref().unwrap_or("");
    let items_selector = if selector.is_empty() { "[]".to_string() } else { format!("{}[]", selector) };
    let document = parse_file(&self.file);
    let (labels, context_values, selector_display) = self.resolve_labels(&document, by, selector, &items_selector);

    let context_hint = if self.context.is_empty() {
      format!("\n\n  {DIM}Add --context \".field\" to show additional fields alongside values{RESET}")
    } else {
      String::new()
    };

    eprintln!("{RED}Error:{RESET} --order is required when using --by");
    eprintln!();
    eprintln!("  {BOLD}Current values (by {by}):{RESET}");

    for (index, label) in labels.iter().enumerate() {
      let context = context_values.get(index).map(|c| c.as_slice()).unwrap_or(&[]);

      eprintln!("    {}", self.format_label_line(index, label, context));
    }

    let csv: String = labels.join(",");

    eprintln!("{context_hint}");
    eprintln!();
    eprintln!("  {BOLD}To sort alphabetically:{RESET}");
    eprintln!("    yerba sort \"{}\"{selector_display} --by \"{by}\" --order asc", self.file);
    eprintln!("    yerba sort \"{}\"{selector_display} --by \"{by}\" --order desc", self.file);
    eprintln!();
    eprintln!("  {BOLD}To reorder explicitly:{RESET}");
    eprintln!("    yerba sort \"{}\"{selector_display} --by \"{by}\" --order \"{csv}\"", self.file);
    eprintln!();
    eprintln!("  {BOLD}To move individual items:{RESET}");
    eprintln!("    yerba move \"{}\"{selector_display} <item> --before/--after <target>", self.file);

    process::exit(1);
  }

  fn run_sort(self) {
    use super::color::*;

    let selector = self.selector.as_deref().unwrap_or("");

    let sort_fields: Vec<yerba::SortField> = if self.by.is_empty() {
      Vec::new()
    } else {
      self
        .by
        .iter()
        .enumerate()
        .map(|(index, field)| {
          let direction = self.order.get(index).map(|s| s.as_str());

          match direction {
            Some("desc" | "descending") => yerba::SortField::desc(field),
            Some("asc" | "ascending") | None => yerba::SortField::asc(field),
            Some(other) => {
              eprintln!("{RED}Error:{RESET} invalid sort direction \"{other}\". Use \"asc\" or \"desc\"");
              process::exit(1);
            }
          }
        })
        .collect()
    };

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if sort_fields.is_empty() {
        let first_item_selector = if selector.is_empty() { "[0]".to_string() } else { format!("{}[0]", selector) };

        match document.get_value(&first_item_selector) {
          Some(first) if first.is_mapping() => {
            eprintln!("{RED}Error:{RESET} --by is required to sort a sequence of maps");
            eprintln!();

            let selectors = document.selectors();
            let prefix = if selector.is_empty() { "[]." } else { "" };
            let fields: Vec<&String> = selectors
              .iter()
              .filter(|s| {
                let check = if prefix.is_empty() { format!("{}[].", selector) } else { prefix.to_string() };
                s.starts_with(&check) && !s[check.len()..].contains('.') && !s[check.len()..].contains('[')
              })
              .collect();

            if !fields.is_empty() {
              eprintln!("  {BOLD}Available fields:{RESET}");

              for field in &fields {
                let short = field.rsplit_once('.').map(|(_, f)| f).unwrap_or(field);
                eprintln!("    .{short}");
              }
            }

            process::exit(1);
          }

          None if selector.is_empty() => {
            eprintln!("{RED}Error:{RESET} no sequence found at root level in {}", resolved_file);
            eprintln!();
            eprintln!("  {DIM}Specify a selector for the sequence to sort:{RESET}");
            eprintln!("    yerba sort \"{}\" \"<selector>\"", self.file);
            eprintln!();

            super::show_similar_selectors(&resolved_file, &document, "[]");

            process::exit(1);
          }

          _ => {}
        }
      }

      if document.sort_items(selector, &sort_fields, self.case_sensitive).is_ok() {
        output(&resolved_file, &document, self.dry_run);
      }
    }
  }

  fn run_reorder(self) {
    use super::color::*;

    if self.by.len() != 1 {
      eprintln!("{RED}Error:{RESET} explicit --order requires exactly one --by field");

      process::exit(1);
    }

    if self.order.len() != 1 {
      eprintln!("{RED}Error:{RESET} explicit --order must be a single comma-separated list");

      process::exit(1);
    }

    let by = &self.by[0];
    let order = &self.order[0];
    let selector = self.selector.as_deref().unwrap_or("");

    let items_selector = if selector.is_empty() { "[]".to_string() } else { format!("{}[]", selector) };

    let mut document = parse_file(&self.file);
    let mut seen = std::collections::HashSet::new();

    let (labels, _, _) = self.resolve_labels(&document, by, selector, &items_selector);
    let duplicates: Vec<&String> = labels.iter().filter(|label| !seen.insert(label.as_str())).collect();

    if !duplicates.is_empty() {
      eprintln!("{RED}Error:{RESET} --order requires unique values for {by}, but found duplicates");
      eprintln!();
      eprintln!("  {BOLD}Duplicate values:{RESET}");

      for label in &duplicates {
        eprintln!("    {label}");
      }

      eprintln!();
      eprintln!("  {DIM}Use \"yerba sort\" with --by instead to sort by field, or");
      eprintln!("  choose a --by field with unique values (e.g. \".id\"){RESET}");

      process::exit(1);
    }

    let values_with_commas: Vec<&String> = labels.iter().filter(|l| l.contains(',')).collect();

    if !values_with_commas.is_empty() {
      let selector_display = if selector.is_empty() { String::new() } else { format!(" \"{selector}\"") };

      eprintln!("{RED}Error:{RESET} some values for {by} contain commas, which conflicts with --order parsing");
      eprintln!();
      eprintln!("  {BOLD}Values with commas:{RESET}");

      for label in &values_with_commas {
        eprintln!("    {label}");
      }

      eprintln!();
      eprintln!("  {BOLD}Use yerba move to reorder individual items instead:{RESET}");
      eprintln!("    yerba move \"{}\"{selector_display} <item> --before/--after <target>", self.file);

      process::exit(1);
    }

    let desired_order: Vec<&str> = order.split(',').map(|s| s.trim()).collect();
    let container = if selector.is_empty() { "" } else { selector };

    match document.reorder_items(container, by, &desired_order) {
      Ok(()) => output(&self.file, &document, self.dry_run),
      Err(error) => {
        eprintln!("{RED}Error:{RESET} {}", error);
        process::exit(1);
      }
    }
  }

  fn resolve_labels(&self, document: &yerba::Document, by: &str, selector: &str, items_selector: &str) -> (Vec<String>, Vec<Vec<String>>, String) {
    use super::color::*;

    let items = document.get_values(items_selector);

    if items.is_empty() {
      if selector.is_empty() {
        eprintln!("{RED}Error:{RESET} no sequence found at root level");
        eprintln!();
        eprintln!("  {DIM}If the file is a map, specify which sequence to sort:{RESET}");
        eprintln!("    yerba sort \"{}\" \"<selector>\" --by \"{by}\" --order asc", self.file);
        eprintln!();

        super::show_similar_selectors(&self.file, document, "[]");
      } else {
        eprintln!("{RED}Error:{RESET} no sequence found at selector: {selector}");

        super::show_similar_selectors(&self.file, document, selector);
      }

      process::exit(1);
    }

    let by_is_scalar = by == ".";

    if !by_is_scalar {
      let by_selector = if selector.is_empty() {
        format!("[].{}", by.strip_prefix('.').unwrap_or(by))
      } else {
        format!("{}[].{}", selector, by.strip_prefix('.').unwrap_or(by))
      };

      if !document.exists(&by_selector) {
        eprintln!("{RED}Error:{RESET} field \"{by}\" not found in items");

        super::show_similar_selectors(&self.file, document, &by_selector);

        process::exit(1);
      }
    }

    let labels: Vec<String> = items
      .iter()
      .map(|item| {
        if by_is_scalar {
          match item {
            yaml_serde::Value::String(string) => string.clone(),
            _ => serde_json::to_string(&yerba::json::yaml_to_json(item)).unwrap_or_default(),
          }
        } else {
          let field = by.strip_prefix('.').unwrap_or(by);

          yerba::json::resolve_select_field(item, field).as_str().unwrap_or("").to_string()
        }
      })
      .collect();

    let context_values: Vec<Vec<String>> = items
      .iter()
      .map(|item| {
        self
          .context
          .iter()
          .map(|context| {
            let field = context.strip_prefix('.').unwrap_or(context);

            let value = yerba::json::resolve_select_field(item, field);

            match &value {
              serde_json::Value::String(string) => string.clone(),
              serde_json::Value::Null => String::new(),
              _ => serde_json::to_string(&value).unwrap_or_default(),
            }
          })
          .collect()
      })
      .collect();

    let selector_display = if selector.is_empty() { String::new() } else { format!(" \"{selector}\"") };

    (labels, context_values, selector_display)
  }

  fn format_label_line(&self, index: usize, label: &str, context: &[String]) -> String {
    use super::color::*;

    if context.is_empty() {
      format!("{DIM}[{index}]{RESET} {label}")
    } else {
      let context = context.iter().filter(|c| !c.is_empty()).cloned().collect::<Vec<_>>().join(", ");

      if context.is_empty() {
        format!("{DIM}[{index}]{RESET} {label}")
      } else {
        format!("{DIM}[{index}]{RESET} {label}  {DIM}{context}{RESET}")
      }
    }
  }
}
