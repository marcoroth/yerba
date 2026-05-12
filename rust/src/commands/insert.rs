use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_files, run_op};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba insert config.yml "database.ssl" true
    yerba insert config.yml "database.ssl" true --after "host"
    yerba insert config.yml "database.ssl" true --before "port"
    yerba insert config.yml "tags" "yaml"
    yerba insert config.yml "tags" "yaml" --at 0
    yerba insert config.yml "tags" "yaml" --after "ruby"
    yerba insert speakers.yml "" "name: Bob" --after ".name == Alice"
    yerba insert videos.yml "[0].speakers" "Diana" --before ".name == Charlie"
    yerba insert videos.yml "" --from "new_talk.yml" --after ".id == first-talk"
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Insert a new key into a map or item into a sequence",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
  value: Option<String>,
  #[arg(long, help = "Read value from a file (use - for stdin)")]
  from: Option<String>,
  #[arg(long, help = "Insert before this value or condition (e.g. \".name == Alice\")")]
  before: Option<String>,
  #[arg(long, help = "Insert after this value or condition (e.g. \".name == Alice\")")]
  after: Option<String>,
  #[arg(long)]
  at: Option<usize>,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let resolved_value = if let Some(from_path) = self.from {
      if from_path == "-" {
        use std::io::Read;
        let mut buffer = String::new();

        std::io::stdin().read_to_string(&mut buffer).unwrap_or_else(|error| {
          use super::color::*;
          eprintln!("{RED}Error:{RESET} reading stdin: {}", error);

          std::process::exit(1);
        });

        buffer.trim().to_string()
      } else {
        std::fs::read_to_string(&from_path)
          .unwrap_or_else(|error| {
            use super::color::*;
            eprintln!("{RED}Error:{RESET} reading {}: {}", from_path, error);
            std::process::exit(1);
          })
          .trim()
          .to_string()
      }
    } else if let Some(val) = self.value {
      val
    } else {
      use super::color::*;
      eprintln!("{RED}Error:{RESET} either a value argument or --from is required");

      std::process::exit(1);
    };

    let parent_path = self.selector.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

    let position = if let Some(index) = self.at {
      yerba::InsertPosition::At(index)
    } else if let Some(target) = self.before {
      if target.starts_with('.') {
        yerba::InsertPosition::BeforeCondition(target)
      } else {
        yerba::InsertPosition::Before(target)
      }
    } else if let Some(target) = self.after {
      if target.starts_with('.') {
        yerba::InsertPosition::AfterCondition(target)
      } else {
        yerba::InsertPosition::After(target)
      }
    } else {
      yerba::Yerbafile::find()
        .and_then(|yerbafile_path| yerba::Yerbafile::load(&yerbafile_path).ok())
        .and_then(|yerbafile| yerbafile.sort_order_for(&self.file, parent_path))
        .map(yerba::InsertPosition::FromSortOrder)
        .unwrap_or(yerba::InsertPosition::Last)
    };

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);
      let result = document.insert_into(&self.selector, &resolved_value, position.clone());

      run_op(&self.file, &document, result);
      output(&resolved_file, &document, self.dry_run);
    }
  }
}
