mod commands;

use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::Parser;
use indoc::indoc;

const STYLES: Styles = Styles::styled()
  .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
  .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
  .literal(AnsiColor::Cyan.on_default())
  .placeholder(AnsiColor::Yellow.on_default())
  .valid(AnsiColor::Green.on_default());

#[derive(Parser)]
#[command(
  name = "yerba",
  version = yerba::version(),
  styles = STYLES,
  about = "Yerba 🧉 YAML Editing and Refactoring with Better Accuracy",
  arg_required_else_help = true,
  override_usage = "yerba [command] [options]",
  disable_help_subcommand = true,
  after_help = indoc! {r#"
    Examples:
      yerba get config.yml database.host
      yerba get videos.yml "[0].title"
      yerba get videos.yml "[]" --select title,speakers
      yerba get "data/**/videos.yml" "[]" --condition ".kind == keynote" --select "id,title"
      yerba set config.yml database.host 0.0.0.0
      yerba insert config.yml tags yaml --after ruby
      yerba insert speakers.yml "" --from speaker.yml --after ".name == Alice"
      yerba delete config.yml database.pool
      yerba move videos.yml "" ".id == talk-2" --after ".id == talk-1"
      yerba sort-keys config.yml database 'id,host,port,name'
      yerba quote-style "data/**/*.yml" double
      yerba check
      yerba apply
  "#}
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
