use std::sync::LazyLock;

use indoc::indoc;

use super::colorize_examples;
use super::{output, parse_file, resolve_move_indexes, run_op};

static EXAMPLES: LazyLock<String> = LazyLock::new(|| {
  colorize_examples(indoc! {r#"
    yerba move config.yml "tags" "rust" --before "ruby"
    yerba move config.yml "tags" "rust" --after "yaml"
    yerba move config.yml "tags" 2 --to 0
    yerba move videos.yml "" ".id == talk-2" --after ".id == talk-1"
  "#})
});

#[derive(clap::Args)]
#[command(
  about = "Move a sequence item to a new position",
  arg_required_else_help = true,
  after_help = EXAMPLES.as_str()
)]
pub struct Args {
  file: String,
  selector: String,
  #[arg(help = "Item to move: name, index, or condition (e.g. \".id == talk-1\")")]
  item: String,
  #[arg(long, help = "Move before this item or condition (e.g. \".id == talk-2\")")]
  before: Option<String>,
  #[arg(long, help = "Move after this item or condition (e.g. \".id == talk-2\")")]
  after: Option<String>,
  #[arg(long)]
  to: Option<usize>,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let mut document = parse_file(&self.file);
    let (from_index, to_index) = resolve_move_indexes(
      &document,
      &self.selector,
      &self.item,
      self.before,
      self.after,
      self.to,
      |document, path, reference| document.resolve_sequence_index(path, reference),
    );

    let result = document.move_item(&self.selector, from_index, to_index);
    run_op(&self.file, &document, result);
    output(&self.file, &document, self.dry_run);
  }
}
