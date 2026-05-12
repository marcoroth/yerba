use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files, run_op};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba remove config.yml "tags" "rust"
    yerba remove videos.yml "[0].speakers" "Alice"
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Remove an item from a sequence by its value",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
  value: String,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);
      let result = document.remove(&self.selector, &self.value);

      run_op(&self.file, &document, result);
      output(&resolved_file, &document, self.dry_run);
    }
  }
}
