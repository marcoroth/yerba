use std::process;
use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba unique config.yml "tags"
    yerba unique videos.yml --by ".id"
    yerba unique speakers.yml --by ".name"
    yerba unique speakers.yml --by ".name" --remove
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Find or remove duplicate items in a sequence",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  /// Selector (optional — omit for root-level sequence)
  selector: Option<String>,
  /// Field to check for uniqueness (omit for scalar arrays)
  #[arg(long)]
  by: Option<String>,
  /// Remove duplicate items (default: report only)
  #[arg(long)]
  remove: bool,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    use super::color::*;

    let selector = self.selector.as_deref().unwrap_or("");
    let by = self.by.as_deref().unwrap_or(".");

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      match document.unique(selector, by, self.remove) {
        Ok(duplicates) => {
          let noun = if duplicates.len() == 1 { "duplicate" } else { "duplicates" };

          if duplicates.is_empty() {
            eprintln!("{GREEN}No duplicates found{RESET} in {}", resolved_file);
          } else if self.remove {
            eprintln!("{YELLOW}Removed {} {noun}{RESET} from {}", duplicates.len(), resolved_file);

            for duplicate in &duplicates {
              eprintln!("  {DIM}line {}: {} == {}{RESET}", duplicate.line, by, duplicate.value);
            }

            output(&resolved_file, &document, self.dry_run);
          } else {
            eprintln!("{RED}Found {} {noun}{RESET} in {}", duplicates.len(), resolved_file);

            for duplicate in &duplicates {
              eprintln!("  {DIM}line {}: {} == {}{RESET}", duplicate.line, by, duplicate.value);
            }

            process::exit(1);
          }
        }

        Err(error) => {
          eprintln!("{RED}Error:{RESET} {}", error);
          process::exit(1);
        }
      }
    }
  }
}
