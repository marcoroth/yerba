use super::run_yerbafile;

#[derive(clap::Args)]
#[command(about = "Apply Yerbafile rules to all matching files (or specific files)")]
pub struct Args {
  /// Specific files to apply rules to (applies to all if omitted)
  files: Vec<String>,
}

impl Args {
  pub fn run(self) {
    run_yerbafile(true, self.files);
  }
}
