use indoc::indoc;

use super::{output, parse_file, run_op};

#[derive(clap::Args)]
#[command(
  about = "Remove an item from a sequence by its value",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba remove config.yml tags rust
      yerba remove videos.yml "[0].speakers" Alice
  "#}
)]
pub struct Args {
  file: String,
  path: String,
  value: String,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let mut document = parse_file(&self.file);
    run_op(|| document.remove(&self.path, &self.value));
    output(&self.file, &document, self.dry_run);
  }
}
