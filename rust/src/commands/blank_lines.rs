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
    yerba blank-lines event.yml "" 1 --before "schedule,speakers"
    yerba blank-lines event.yml "" 1 --after "location" --skip-empty
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
  /// Comma-separated map keys to separate from what precedes them
  #[arg(long)]
  before: Option<String>,
  /// Comma-separated map keys to separate from what follows them
  #[arg(long)]
  after: Option<String>,
  /// Leave entries alone when their value is null, an empty collection, or an empty string
  #[arg(long)]
  skip_empty: bool,
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
      use super::ui;

      eprintln!("{} expected a number for blank line count, got '{}'", ui::failure("Error:"), self.first);

      std::process::exit(1);
    };

    let options = yerba::BlankLineOptions {
      before: split_keys(self.before.as_deref()),
      after: split_keys(self.after.as_deref()),
      skip_empty: self.skip_empty,
    };

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if document.enforce_blank_lines_with(selector, count, &options).is_ok() {
        output(&resolved_file, &document, self.dry_run);
      }
    }
  }
}

fn split_keys(keys: Option<&str>) -> Vec<String> {
  keys.map(|keys| keys.split(',').map(|key| key.trim().to_string()).collect()).unwrap_or_default()
}
