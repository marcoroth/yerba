use indoc::indoc;

use super::{output, parse_file, run_op};

#[derive(clap::Args)]
#[command(
  about = "Update an existing value at a path (preserves quote style)",
  arg_required_else_help = true,
  after_help = indoc! {r#"
    Examples:
      yerba set config.yml database.host 0.0.0.0
      yerba set config.yml database.host 0.0.0.0 --if-exists
      yerba set config.yml database.host 0.0.0.0 --condition '.port == 5432'
      yerba set videos.yml "[0].title" "New Title"
      yerba set "data/**/event.yml" website "" --if-exists
  "#}
)]
pub struct Args {
  file: String,
  path: String,
  value: String,
  #[arg(long)]
  if_exists: bool,
  #[arg(long)]
  if_missing: bool,
  #[arg(long)]
  condition: Option<String>,
  #[arg(long)]
  dry_run: bool,
}

impl Args {
  pub fn run(self) {
    let mut document = parse_file(&self.file);
    let parent_path = self.path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

    let should_set = if self.if_exists {
      document.exists(&self.path)
    } else if self.if_missing {
      !document.exists(&self.path)
    } else if let Some(condition) = &self.condition {
      document.evaluate_condition(parent_path, condition)
    } else {
      true
    };

    if should_set {
      run_op(|| document.set(&self.path, &self.value));
    }

    output(&self.file, &document, self.dry_run);
  }
}
