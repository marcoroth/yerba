use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba quote-style config.yml --values double
    yerba quote-style config.yml --keys plain
    yerba quote-style config.yml --keys plain --values double
    yerba quote-style config.yml "[].speakers" --values plain
    yerba quote-style "data/**/*.yml" --keys plain --values double
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Enforce a consistent quote style on keys and/or values",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  /// Selector to scope the operation (optional — omit for whole file)
  selector: Option<String>,
  /// Quote style for keys
  #[arg(long)]
  keys: Option<yerba::KeyStyle>,
  /// Quote style for values
  #[arg(long)]
  values: Option<yerba::QuoteStyle>,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    if self.values.is_none() && self.keys.is_none() {
      use super::color::*;
      eprintln!("{RED}Error:{RESET} specify --values, --keys, or both");
      std::process::exit(1);
    }

    let selector = self.selector.as_deref().filter(|s| !s.is_empty());

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if let Some(style) = &self.keys {
        let _ = document.enforce_key_style(style, selector);
      }

      if let Some(style) = &self.values {
        if let Ok(warnings) = document.enforce_quotes_at(style, selector) {
          for warning in warnings {
            use super::color::*;

            eprintln!("{YELLOW}Warning:{RESET} {} — {}", resolved_file, warning);
          }
        }
      }

      output(&resolved_file, &document, self.dry_run);
    }
  }
}
