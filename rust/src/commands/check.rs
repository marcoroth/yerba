use super::run_yerbafile;

#[derive(clap::Args)]
#[command(about = "Check if files match Yerbafile rules (exits 1 if changes needed)")]
pub struct Args {
  /// Specific files to check (checks all if omitted)
  files: Vec<String>,
}

impl Args {
  pub fn run(self) {
    run_yerbafile(false, self.files);
  }
}
