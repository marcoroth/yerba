use std::process;
use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::parse_file;

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba location config.yml "database.host"
    yerba location videos.yml "[0]"
    yerba location videos.yml "[0].title"
    yerba location videos.yml "[0].speakers"
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Show the location (line, column, offset) of a selector",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
}

impl Args {
  pub fn run(self) {
    use super::color::*;

    let document = parse_file(&self.file);
    let info = document.get_node_info(&self.selector);

    if info.location.start_line == 0 && info.location.end_line == 0 {
      eprintln!("{RED}Error:{RESET} selector \"{}\" not found in {}", self.selector, self.file);

      super::show_similar_selectors(&self.file, &document, &self.selector);
      process::exit(1);
    }

    let json = serde_json::json!({
      "selector": self.selector,
      "file": self.file,
      "start_line": info.location.start_line,
      "start_column": info.location.start_column,
      "end_line": info.location.end_line,
      "end_column": info.location.end_column,
      "start_offset": info.location.start_offset,
      "end_offset": info.location.end_offset,
    });

    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
  }
}
