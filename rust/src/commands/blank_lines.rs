use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba blank-lines videos.yml 1
    yerba blank-lines videos.yml "[]" 1
    yerba blank-lines videos.yml "[].speakers" 1
    yerba blank-lines config.yml "tags" 0
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Enforce blank lines between sequence entries",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  /// Selector or count (if a number, treated as count for root-level sequence)
  first: String,
  /// Count (when selector is provided as first positional)
  second: Option<usize>,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let (selector, count) = if let Some(count) = self.second {
      (self.first.as_str(), count)
    } else if let Ok(count) = self.first.parse::<usize>() {
      ("", count)
    } else {
      use super::color::*;

      eprintln!("{RED}Error:{RESET} expected a number for blank line count, got '{}'", self.first);

      std::process::exit(1);
    };

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if document.enforce_blank_lines(selector, count).is_ok() {
        output(&resolved_file, &document, self.dry_run);
      }
    }
  }
}
