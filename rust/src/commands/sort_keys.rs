use std::process;

use indoc::indoc;

use super::{output, parse_file, resolve_files};

#[derive(clap::Args)]
#[command(
  about = "Sort keys in a map by a predefined order (aborts on unknown keys)",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba sort-keys config.yml "database" "host,port,name,pool"
      yerba sort-keys "data/**/event.yml" "" "id,title,kind,location"
      yerba sort-keys "data/**/videos.yml" "[]" "id,title,speakers"
      yerba sort-keys config.yml "database" "host,port" --dry-run
  "#}
)]
pub struct Args {
  file: String,
  selector: String,
  /// Comma-separated key order
  order: String,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let key_order: Vec<&str> = self.order.split(',').collect();
    let files = resolve_files(&self.file);

    let mut has_errors = false;

    for resolved_file in &files {
      let document = parse_file(resolved_file);

      if let Err(error) = document.validate_sort_keys(&self.selector, &key_order) {
        use super::color::*;
        eprintln!("{RED}Error in {}{RESET}: {}", resolved_file, error);
        has_errors = true;
      }
    }

    if has_errors {
      process::exit(1);
    }

    for resolved_file in &files {
      let mut document = parse_file(resolved_file);

      if document.sort_keys(&self.selector, &key_order).is_ok() {
        output(resolved_file, &document, self.dry_run);
      }
    }
  }
}
