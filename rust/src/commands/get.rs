use std::process;

use indoc::indoc;

use super::{parse_file, resolve_files};

#[derive(clap::Args)]
#[command(
  about = "Get values at a path (single or multi-value with [] brackets)",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba get config.yml database.host
      yerba get config.yml database.host --condition '.port == 5432'
      yerba get videos.yml "[].title"
      yerba get "data/**/videos.yml" "[].speakers[].name"
      yerba get videos.yml "[0].title"
  "#}
)]
pub struct Args {
  file: String,
  path: String,
  #[arg(long)]
  condition: Option<String>,
}

impl Args {
  pub fn run(self) {
    for resolved_file in resolve_files(&self.file) {
      let document = parse_file(&resolved_file);

      if let Some(condition) = &self.condition {
        let parent_path = self.path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, condition) {
          continue;
        }
      }

      let values = document.get_all(&self.path);

      if values.is_empty() && !self.path.contains('[') && !document.exists(&self.path) {
        use super::color::*;
        eprintln!("{RED}Error:{RESET} path not found: {}", self.path);
        process::exit(1);
      }

      for value in values {
        println!("{}", value);
      }
    }
  }
}
