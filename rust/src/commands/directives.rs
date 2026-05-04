use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba directives config.yml --ensure
    yerba directives config.yml --remove
    yerba directives "data/**/*.yml" --ensure
    yerba directives config.yml --ensure --dry-run
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Add or remove the document start marker (---)",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  /// Add --- if missing
  #[arg(long)]
  ensure: bool,
  /// Remove --- if present
  #[arg(long)]
  remove: bool,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    if !self.ensure && !self.remove {
      use super::color::*;
      eprintln!("{RED}Error:{RESET} specify --ensure or --remove");
      std::process::exit(1);
    }

    if self.ensure && self.remove {
      use super::color::*;
      eprintln!("{RED}Error:{RESET} --ensure and --remove are mutually exclusive");
      std::process::exit(1);
    }

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if self.ensure {
        let _ = document.ensure_directives();
      } else if self.remove {
        let _ = document.remove_directives();
      }

      output(&resolved_file, &document, self.dry_run);
    }
  }
}
