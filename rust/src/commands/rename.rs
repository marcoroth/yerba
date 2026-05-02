use indoc::indoc;

use super::{output, parse_file, run_op};

#[derive(clap::Args)]
#[command(
  about = "Rename a key in a map (preserves value and position)",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba rename config.yml database.host database.hostname
      yerba rename config.yml database.host hostname
      yerba rename config.yml database.host settings.db_host
      yerba rename videos.yml "[0].old_name" "[0].name"
  "#}
)]
pub struct Args {
  file: String,
  source: String,
  destination: String,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let mut document = parse_file(&self.file);
    run_op(|| document.rename(&self.source, &self.destination));
    output(&self.file, &document, self.dry_run);
  }
}
