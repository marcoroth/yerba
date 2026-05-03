use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, run_op};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba delete config.yml "database.pool"
    yerba delete videos.yml "[0].description"
    yerba delete config.yml "database.pool" --dry-run
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Delete a key and its value from a map",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let mut document = parse_file(&self.file);
    run_op(|| document.delete(&self.selector));
    output(&self.file, &document, self.dry_run);
  }
}
