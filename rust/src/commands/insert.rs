use super::ui;
use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{default_insert_position, output, parse_file, resolve_files, run_op};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba insert config.yml "database.ssl" true --plain
    yerba insert config.yml "database.ssl" true --plain --after "host"
    yerba insert config.yml "database.ssl" true --plain --before "port"
    yerba insert config.yml "tags" "yaml"
    yerba insert config.yml "tags" "yaml" --at 0
    yerba insert config.yml "tags" "yaml" --after "ruby"
    yerba insert speakers.yml "" "name: Bob" --plain --after ".name == Alice"
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
  #[arg(allow_hyphen_values = true)]
  value: Option<String>,
  #[arg(long, help = "Read value from a file (use - for stdin)")]
  from: Option<String>,
  #[arg(long, help = "Insert before this value or condition (e.g. \".name == Alice\")")]
  before: Option<String>,
  #[arg(long, help = "Insert after this value or condition (e.g. \".name == Alice\")")]
  after: Option<String>,
  #[arg(long)]
  at: Option<usize>,
  #[arg(long, help = "Write the value as raw YAML, for literals like null, true or 42 and for map or sequence fragments")]
  plain: bool,
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
          use super::ui;
          eprintln!("{} reading stdin: {}", ui::failure("Error:"), error);

          std::process::exit(1);
        });

        buffer.trim().to_string()
      } else {
        std::fs::read_to_string(&from_path)
          .unwrap_or_else(|error| {
            eprintln!("{} reading {}: {}", ui::failure("Error:"), from_path, error);
            std::process::exit(1);
          })
          .trim()
          .to_string()
      }
    } else if let Some(value) = self.value {
      if self.plain {
        value
      } else {
        yerba::quote_scalar(&value)
      }
    } else {
      eprintln!("{} either a value argument or --from is required", ui::failure("Error:"));

      std::process::exit(1);
    };

    let position = if let Some(index) = self.at {
      yerba::InsertPosition::At(index)
    } else if let Some(target) = self.before {
      yerba::InsertPosition::before(target)
    } else if let Some(target) = self.after {
      yerba::InsertPosition::after(target)
    } else {
      default_insert_position(&self.file, &self.selector)
    };

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);
      let result = document.insert_into(&self.selector, &resolved_value, position.clone());

      run_op(&self.file, &document, result);
      output(&resolved_file, &document, self.dry_run);
    }
  }
}
