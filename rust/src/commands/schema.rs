use std::process;

use super::resolve_files;
use super::ui;

#[derive(clap::Args)]
#[command(
  about = "Validate YAML files against a JSON schema",
  after_help = colorize_examples(indoc::indoc! {r#"
    yerba schema data/speakers.yml --schema lib/schemas/speaker_schema.json
    yerba schema "data/**/videos.yml" --schema lib/schemas/video_schema.json
    yerba schema data/config.yml --schema schema.json --selector "database"
  "#})
)]
pub struct Args {
  /// YAML file(s) to validate (supports globs)
  file: String,
  /// Path to the JSON schema file
  #[arg(long)]
  schema: String,
  /// Selector to scope validation (e.g. "[]", "tiers[]")
  #[arg(long)]
  selector: Option<String>,
}

fn colorize_examples(text: &str) -> String {
  super::colorize_examples(text)
}

impl Args {
  pub fn run(self) {
    let schema = match yerba::schema::load_schema(&self.schema) {
      Ok(schema) => schema,
      Err(error) => {
        eprintln!("{} {}", ui::failure("Error:"), error);
        process::exit(1);
      }
    };

    let files = resolve_files(&self.file);

    if files.is_empty() {
      eprintln!("{} no files matching: {}", ui::failure("Error:"), self.file);
      process::exit(1);
    }

    let mut has_errors = false;
    let selector = self.selector.as_deref();

    for file in &files {
      let document = match yerba::Document::parse_file(file) {
        Ok(document) => document,
        Err(error) => {
          eprintln!("  {} {} {} {}", ui::failure("error:"), ui::subtle("—"), file, error);
          has_errors = true;
          continue;
        }
      };

      let errors = document.validate_schema(&schema, false, selector);

      if errors.is_empty() {
        continue;
      }

      has_errors = true;

      let relative = file.strip_prefix("./").unwrap_or(file);
      eprintln!("  {} {relative}", ui::failure("error:"));

      for error in &errors {
        eprintln!("    {error}");
      }

      eprintln!();
    }

    if has_errors {
      process::exit(1);
    } else {
      eprintln!("{}", ui::success(format!("All {} files valid.", files.len())));
    }
  }
}
