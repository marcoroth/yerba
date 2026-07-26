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
    yerba set "data/**/event.yml" "website" "" --if-exists
    yerba set videos.yml "[].description" "" --all
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
  value: String,
  #[arg(long)]
  if_exists: bool,
  #[arg(long)]
  if_missing: bool,
  #[arg(long)]
  condition: Option<String>,
  #[arg(long)]
  all: bool,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let parent_path = self.selector.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

    if let Some(condition) = &self.condition {
      if let Err(error) = yerba::validate_condition(condition) {
        use super::color::*;

        eprintln!("{RED}Error:{RESET} {}", error);

        std::process::exit(1);
      }
    }

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      let should_set = if self.if_exists {
        document.exists(&self.selector)
      } else if self.if_missing {
        !document.exists(&self.selector)
      } else if let Some(condition) = &self.condition {
        document.evaluate_condition(parent_path, condition).unwrap_or(false)
      } else {
        true
      };

      if should_set {
        let result = if self.all {
          document.set_all(&self.selector, &self.value)
        } else {
          document.set(&self.selector, &self.value)
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
