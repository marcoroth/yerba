mod commands;

use std::sync::LazyLock;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::Parser;
use indoc::indoc;

const STYLES: Styles = Styles::styled()
  .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
  .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
  .literal(AnsiColor::Cyan.on_default())
  .placeholder(AnsiColor::Yellow.on_default())
  .valid(AnsiColor::Green.on_default());

static HELP: LazyLock<String> = LazyLock::new(|| {
  commands::colorize_help(indoc! {r#"
    Selectors:
      key                  A single key          "database.host"
      key.nested           Nested key path       "database.settings.pool"
      []                   All items in array    "[].title"
      [N]                  Item at index         "[0].title"
      *                    All values in a map   "database.*"
      [].key[].nested      Nested array access   "[].speakers[].name"

    Conditions:
      .key == value        Equality              ".kind == keynote"
      .key != value        Inequality            ".status != draft"
      .key contains val    Substring or member   ".title contains Ruby"
      .key not_contains    Negated contains      ".title not_contains test"

    Yerbafile:
      yerba init           Create a new Yerbafile in the current directory
      yerba check          Check if all files match the rules (exits 1 if not)
      yerba check <file>   Check a specific file against matching rules
      yerba apply          Apply all rules and write changes
      yerba apply <file>   Apply rules to a specific file

    Examples:
      yerba get config.yml "database.host"
      yerba get videos.yml "[0].title"
      yerba get videos.yml "[]" --select ".title,.speakers"
      yerba get "data/**/videos.yml" "[]" --condition ".kind == keynote" --select ".id,.title"
      yerba set config.yml "database.host" "0.0.0.0"
      yerba insert config.yml "tags" "yaml" --after "ruby"
      yerba insert speakers.yml "" --from "speaker.yml" --after ".name == Alice"
      yerba delete config.yml "database.pool"
      yerba move videos.yml "" ".id == talk-2" --after ".id == talk-1"
      yerba sort-keys config.yml "database" "id,host,port,name"
      yerba quote-style "data/**/*.yml" --values double
      yerba sort videos.yml "[]" --by ".id" --order "talk-3,talk-1,talk-2"
      yerba schema "data/**/*.yml" --schema schema.json
      yerba selectors videos.yml
      yerba location videos.yml "[0].title"
  "#})
});

#[derive(Parser)]
#[command(
  name = "yerba",
  version = yerba::version(),
  disable_version_flag = true,
  styles = STYLES,
  about = "Yerba 🧉 YAML Editing and Refactoring with Better Accuracy",
  arg_required_else_help = true,
  override_usage = "\x1b[1myerba\x1b[0m <command> <file> <selector> [options]",
  disable_help_subcommand = true,
  after_help = HELP.as_str()
)]
#[allow(clippy::upper_case_acronyms)]
struct CLI {
  #[command(subcommand)]
  command: commands::Command,
}

fn main() {
  let cli = CLI::parse();
  cli.command.run();
}
