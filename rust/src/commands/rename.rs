use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files, run_op};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba rename config.yml "database.host" "database.hostname"
    yerba rename videos.yml "[0].old_name" "[0].name"
    yerba rename config.yml "database.host" "settings.db_host"  # moves the key to another map
    yerba rename config.yml "database.host" "hostname"          # moves the key to the document root
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Rename a key, or move it to another map (the destination is a full selector path)",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
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
    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);
      let result = document.rename(&self.source, &self.destination);

      run_op(&self.file, &document, result);
      output(&resolved_file, &document, self.dry_run);
    }
  }
}
