use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files, run_op_with_hint};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba set config.yml "database.host" "0.0.0.0"
    yerba set config.yml "database.host" "0.0.0.0" --if-exists
    yerba set config.yml "database.host" "0.0.0.0" --condition ".port == 5432"
    yerba set videos.yml "[0].title" "New Title"
    yerba set videos.yml "[].title" "New Title" --condition ".id == talk-1"
    yerba set "data/**/event.yml" "website" "" --if-exists
    yerba set videos.yml "[].description" "" --all
    yerba set config.yml "database.replica" null --plain
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Update an existing value at a path (preserves quote style)",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
  #[arg(allow_hyphen_values = true)]
  value: String,
  #[arg(long)]
  if_exists: bool,
  #[arg(long)]
  if_missing: bool,
  #[arg(long)]
  condition: Option<String>,
  #[arg(long)]
  all: bool,
  #[arg(long, help = "Write the value unquoted, for YAML literals like null, true or 42")]
  plain: bool,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let parent_path = self.selector.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

    if let Some(condition) = &self.condition {
      if let Err(error) = yerba::validate_condition(condition) {
        use super::ui;

        eprintln!("{} {}", ui::failure("Error:"), error);

        std::process::exit(1);
      }
    }

    let leaf = self.selector.rsplit_once('.').map(|(_, leaf)| leaf).unwrap_or(&self.selector);
    let per_item = yerba::Selector::parse(parent_path).has_brackets();

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      let should_set = if self.if_exists {
        document.exists(&self.selector)
      } else if self.if_missing {
        !document.exists(&self.selector)
      } else if let Some(condition) = &self.condition {
        per_item || document.evaluate_condition(parent_path, condition).unwrap_or(false)
      } else {
        true
      };

      if should_set {
        let result = match &self.condition {
          Some(condition) if per_item => document.set_where(parent_path, leaf, &self.value, condition, self.all, self.plain).map(|_| ()),
          _ if self.all && self.plain => document.set_all_plain(&self.selector, &self.value),
          _ if self.all => document.set_all(&self.selector, &self.value),
          _ if self.plain => document.set_plain(&self.selector, &self.value),
          _ => document.set(&self.selector, &self.value),
        };

        run_op_with_hint(
          &self.file,
          &document,
          result,
          Some("Use --if-exists to skip files where the selector is missing"),
        );
      }

      output(&resolved_file, &document, self.dry_run);
    }
  }
}
