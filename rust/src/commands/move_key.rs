use std::process;
use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_move_indexes, run_op};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba move-key config.yml "database.name" --to 0
    yerba move-key config.yml "database.pool" --before "database.host"
    yerba move-key config.yml "database.pool" --after "database.name"
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Move a key to a new position within a map",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
  #[arg(long, help = "Move before this key")]
  before: Option<String>,
  #[arg(long, help = "Move after this key")]
  after: Option<String>,
  #[arg(long)]
  to: Option<usize>,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let (parent_path, key) = self.selector.rsplit_once('.').unwrap_or(("", &self.selector));

    let mut document = parse_file(&self.file);

    let before_key = self.before.map(|target| {
      let (target_parent, target_key) = target.rsplit_once('.').unwrap_or(("", &target));

      if target_parent != parent_path {
        use super::color::*;
        eprintln!(
          "{RED}Error:{RESET} cannot move key across different maps ({} \u{2192} {})\n\n  Use 'yerba rename' to relocate keys to a different path.",
          self.selector, target
        );

        process::exit(1);
      }

      target_key.to_string()
    });

    let after_key = self.after.map(|target| {
      let (target_parent, target_key) = target.rsplit_once('.').unwrap_or(("", &target));

      if target_parent != parent_path {
        use super::color::*;
        eprintln!(
          "{RED}Error:{RESET} cannot move key across different maps ({} \u{2192} {})\n\n  Use 'yerba rename' to relocate keys to a different path.",
          self.selector, target
        );

        process::exit(1);
      }

      target_key.to_string()
    });

    let (from_index, to_index) = resolve_move_indexes(
      &document,
      parent_path,
      key,
      before_key,
      after_key,
      self.to,
      |document, parent_path, reference| document.resolve_key_index(parent_path, reference),
    );

    let result = document.move_key(parent_path, from_index, to_index);
    run_op(&self.file, &document, result);
    output(&self.file, &document, self.dry_run);
  }
}
