use indoc::indoc;

use super::{output, parse_file, run_op};

#[derive(clap::Args)]
#[command(
  about = "Delete a key and its value from a map",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba delete config.yml database.pool
      yerba delete videos.yml "[0].description"
      yerba delete config.yml database.pool --dry-run
  "#}
)]
pub struct Args {
  file: String,
  path: String,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let mut document = parse_file(&self.file);
    run_op(|| document.delete(&self.path));
    output(&self.file, &document, self.dry_run);
  }
}
