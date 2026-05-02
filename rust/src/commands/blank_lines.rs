use indoc::indoc;

use super::{output, parse_file, resolve_files};

#[derive(clap::Args)]
#[command(
  about = "Enforce blank lines between sequence entries",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba blank-lines videos.yml "" 1
      yerba blank-lines "data/**/videos.yml" "[]" 1
      yerba blank-lines videos.yml "[].speakers" 1
      yerba blank-lines config.yml "tags" 0
  "#}
)]
pub struct Args {
  file: String,
  selector: String,
  /// Number of blank lines between entries (0 = no blanks, 1 = one empty line)
  count: usize,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if document.enforce_blank_lines(&self.selector, self.count).is_ok() {
        output(&resolved_file, &document, self.dry_run);
      }
    }
  }
}
