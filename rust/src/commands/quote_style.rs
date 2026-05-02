use indoc::indoc;

use super::{output, parse_file, resolve_files};

#[derive(clap::Args)]
#[command(
  about = "Enforce a consistent quote style on values, keys, or both",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba quote-style config.yml double
      yerba quote-style config.yml plain --keys
      yerba quote-style config.yml double --all
      yerba quote-style config.yml single --path "database.host"
      yerba quote-style videos.yml plain --path "[].speakers"
  "#}
)]
pub struct Args {
  file: String,
  /// Quote style
  style: yerba::QuoteStyle,
  /// Scope to a specific path
  #[arg(long)]
  path: Option<String>,
  /// Apply to keys only
  #[arg(long)]
  keys: bool,
  /// Apply to both keys and values
  #[arg(long)]
  all: bool,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let dot_path = self.path.as_deref();

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if self.keys {
        let _ = document.enforce_key_style(&self.style, dot_path);
      } else if self.all {
        let _ = document.enforce_key_style(&self.style, dot_path);
        let _ = document.enforce_quotes_at(&self.style, dot_path);
      } else {
        let _ = document.enforce_quotes_at(&self.style, dot_path);
      }

      output(&resolved_file, &document, self.dry_run);
    }
  }
}
