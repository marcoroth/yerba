use indoc::indoc;

use super::{output, parse_file, resolve_files};

#[derive(clap::Args)]
#[command(
  about = "Sort items in a sequence by field(s)",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba sort config.yml tags
      yerba sort videos.yml "" --by title
      yerba sort videos.yml "" --by "date:desc,title"
      yerba sort videos.yml "[].speakers"
      yerba sort videos.yml "[].speakers" --by name
      yerba sort videos.yml "" --by "kind,date:desc,title" --dry-run
  "#}
)]
pub struct Args {
  file: String,
  path: String,
  /// Comma-separated sort fields, optionally with :desc (e.g. "date:desc,title")
  #[arg(long)]
  by: Option<String>,
  /// Case-sensitive sort (default: case-insensitive)
  #[arg(long)]
  case_sensitive: bool,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let sort_fields = self.by.as_deref().map(yerba::SortField::parse_list).unwrap_or_default();

    for resolved_file in resolve_files(&self.file) {
      let mut document = parse_file(&resolved_file);

      if document
        .sort_items(&self.path, &sort_fields, self.case_sensitive)
        .is_ok()
      {
        output(&resolved_file, &document, self.dry_run);
      }
    }
  }
}
