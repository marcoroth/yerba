pub mod apply;
pub mod blank_lines;
pub mod check;
pub mod delete;
pub mod get;
pub mod init;
pub mod insert;
pub mod mate;
pub mod move_item;
pub mod move_key;
pub mod quote_style;
pub mod remove;
pub mod rename;
pub mod set;
pub mod sort;
pub mod sort_keys;
pub mod version;

use std::fs;
use std::process;

use clap::Subcommand;

pub(crate) mod color {
  pub const GREEN: &str = "\x1b[32m";
  pub const RED: &str = "\x1b[31m";
  pub const YELLOW: &str = "\x1b[33m";
  pub const DIM: &str = "\x1b[2m";
  pub const BOLD: &str = "\x1b[1m";
  pub const RESET: &str = "\x1b[0m";
}

// Compile-time ANSI macros for use in concat!() / clap attributes (used in main.rs)
#[allow(unused_macros)]
macro_rules! h {
  () => {
    "\x1b[1;32m"
  };
} // header (bold green)
#[allow(unused_macros)]
macro_rules! b {
  () => {
    "\x1b[1m"
  };
} // bold
#[allow(unused_macros)]
macro_rules! c {
  () => {
    "\x1b[36m"
  };
} // cyan
#[allow(unused_macros)]
macro_rules! d {
  () => {
    "\x1b[2m"
  };
} // dim
#[allow(unused_macros)]
macro_rules! r {
  () => {
    "\x1b[0m"
  };
} // reset
#[allow(unused_imports)]
pub(crate) use {b, c, d, h, r};

pub(crate) fn colorize_examples(input: &str) -> String {
  colorize_help(&format!("Examples:\n{}", input.trim()))
}

pub(crate) fn colorize_help(input: &str) -> String {
  use color::*;

  let mut output = String::new();

  for line in input.lines() {
    let trimmed = line.trim();

    if trimmed.is_empty() {
      output.push('\n');
      continue;
    }

    if trimmed.ends_with(':') && !trimmed.contains(' ') {
      output.push_str(&format!("{GREEN}{BOLD}{trimmed}{RESET}\n"));
      continue;
    }

    if trimmed.starts_with("yerba ") {
      let mut parts = trimmed.splitn(3, ' ');

      match (parts.next(), parts.next(), parts.next()) {
        (Some(cmd), Some(sub), Some(rest)) => {
          output.push_str(&format!("  {BOLD}{cmd}{RESET} \x1b[36m{sub}{RESET} {rest}\n"));
        }

        (Some(cmd), Some(sub), None) => {
          output.push_str(&format!("  {BOLD}{cmd}{RESET} \x1b[36m{sub}{RESET}\n"));
        }

        _ => {
          output.push_str(&format!("  {trimmed}\n"));
        }
      }

      continue;
    }

    let mut columns: Vec<&str> = Vec::new();
    let mut rest = trimmed;

    while !rest.is_empty() {
      let column_end = rest.find("  ").unwrap_or(rest.len());
      let column = rest[..column_end].trim();

      if !column.is_empty() {
        columns.push(column);
      }

      if column_end >= rest.len() {
        break;
      }

      rest = rest[column_end..].trim_start();
    }

    if columns.len() == 3 {
      output.push_str(&format!(
        "  \x1b[36m{:<20}{RESET} {:<21} {DIM}{}{RESET}\n",
        columns[0], columns[1], columns[2]
      ));
    } else if columns.len() == 2 {
      output.push_str(&format!("  \x1b[36m{:<20}{RESET} {}\n", columns[0], columns[1]));
    } else {
      output.push_str(&format!("  {trimmed}\n"));
    }
  }

  output.trim_end().to_string()
}

#[derive(Subcommand)]
pub enum Command {
  Get(get::Args),
  Set(set::Args),
  Insert(insert::Args),
  Rename(rename::Args),
  Delete(delete::Args),
  Remove(remove::Args),
  Move(move_item::Args),
  MoveKey(move_key::Args),
  SortKeys(sort_keys::Args),
  Sort(sort::Args),
  QuoteStyle(quote_style::Args),
  BlankLines(blank_lines::Args),
  #[command(about = "Create a new Yerbafile in the current directory")]
  Init,
  #[command(about = "Apply all rules from the Yerbafile and write changes")]
  Apply,
  #[command(about = "Check if all files match Yerbafile rules (exits 1 if not)")]
  Check,
  #[command(about = "Print the yerba version")]
  Version,
  #[command(about = "\u{1f9c9}")]
  Mate,
}

impl Command {
  pub fn run(self) {
    match self {
      Command::Get(args) => args.run(),
      Command::Set(args) => args.run(),
      Command::Insert(args) => args.run(),
      Command::Rename(args) => args.run(),
      Command::Delete(args) => args.run(),
      Command::Remove(args) => args.run(),
      Command::Move(args) => args.run(),
      Command::MoveKey(args) => args.run(),
      Command::SortKeys(args) => args.run(),
      Command::Sort(args) => args.run(),
      Command::QuoteStyle(args) => args.run(),
      Command::BlankLines(args) => args.run(),
      Command::Init => init::run(),
      Command::Apply => apply::run(),
      Command::Check => check::run(),
      Command::Version => version::run(),
      Command::Mate => mate::run(),
    }
  }
}

pub(crate) fn run_yerbafile(write: bool) {
  use color::*;

  let yerbafile_path = yerba::Yerbafile::find().unwrap_or_else(|| {
    eprintln!("{RED}No Yerbafile found.{RESET} Run {BOLD}yerba init{RESET} to create one.");
    process::exit(1);
  });

  let yerbafile = yerba::Yerbafile::load(&yerbafile_path).unwrap_or_else(|error| {
    eprintln!("{RED}Error loading {}:{RESET} {}", yerbafile_path.display(), error);
    process::exit(1);
  });

  eprintln!("🧉 {BOLD}Using{RESET} {}", yerbafile_path.display());

  let results = yerbafile.apply(write);
  let mut has_changes = false;
  let mut has_errors = false;

  for result in &results {
    if let Some(error) = &result.error {
      eprintln!("  {RED}error:{RESET} {} {DIM}—{RESET} {}", result.file, error);
      has_errors = true;
    } else if result.changed {
      if write {
        eprintln!("  {GREEN}updated:{RESET} {}", result.file);
      } else {
        eprintln!("  {YELLOW}would change:{RESET} {}", result.file);
      }

      has_changes = true;
    }
  }

  if !has_changes && !has_errors {
    eprintln!("  {GREEN}All files match the rules.{RESET}");
  }

  if !write && has_changes {
    process::exit(1);
  }

  if has_errors {
    process::exit(1);
  }
}

pub(crate) fn resolve_move_indexes(
  document: &yerba::Document,
  path: &str,
  item: &str,
  before: Option<String>,
  after: Option<String>,
  to: Option<usize>,
  resolve: impl Fn(&yerba::Document, &str, &str) -> Result<usize, yerba::YerbaError>,
) -> (usize, usize) {
  use color::*;

  let from_index = resolve(document, path, item).unwrap_or_else(|error| {
    eprintln!("{RED}Error:{RESET} {}", error);
    process::exit(1);
  });

  let to_index = if let Some(index) = to {
    index
  } else if let Some(target) = &before {
    let target_index = resolve(document, path, target).unwrap_or_else(|error| {
      eprintln!("{RED}Error:{RESET} {}", error);
      process::exit(1);
    });

    if from_index < target_index {
      target_index - 1
    } else {
      target_index
    }
  } else if let Some(target) = &after {
    let target_index = resolve(document, path, target).unwrap_or_else(|error| {
      eprintln!("{RED}Error:{RESET} {}", error);
      process::exit(1);
    });

    if from_index <= target_index {
      target_index
    } else {
      target_index + 1
    }
  } else {
    eprintln!("{RED}Error:{RESET} specify --before, --after, or --to");
    process::exit(1);
  };

  (from_index, to_index)
}

pub(crate) fn resolve_files(pattern: &str) -> Vec<String> {
  use color::*;

  if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
    let paths: Vec<String> = glob::glob(pattern)
      .unwrap_or_else(|error| {
        eprintln!("{RED}Error:{RESET} invalid glob pattern '{}': {}", pattern, error);
        process::exit(1);
      })
      .filter_map(|entry| entry.ok())
      .map(|path| path.to_string_lossy().to_string())
      .collect();

    if paths.is_empty() {
      eprintln!("{RED}Error:{RESET} no files matched pattern: {}", pattern);
      process::exit(1);
    }

    paths
  } else {
    vec![pattern.to_string()]
  }
}

pub(crate) fn parse_file(file: &str) -> yerba::Document {
  use color::*;

  yerba::parse_file(file).unwrap_or_else(|error| {
    match &error {
      yerba::YerbaError::IoError(io_error) => match io_error.kind() {
        std::io::ErrorKind::NotFound => eprintln!("{RED}Error:{RESET} file not found: {}", file),
        std::io::ErrorKind::PermissionDenied => {
          eprintln!("{RED}Error:{RESET} permission denied: {}", file)
        }
        _ => eprintln!("{RED}Error:{RESET} reading {}: {}", file, io_error),
      },
      _ => eprintln!("{RED}Error:{RESET} parsing {}: {}", file, error),
    }

    process::exit(1);
  })
}

pub(crate) fn run_op(operation: impl FnOnce() -> Result<(), yerba::YerbaError>) {
  use color::*;

  operation().unwrap_or_else(|error| {
    eprintln!("{RED}Error:{RESET} {}", error);
    process::exit(1);
  });
}

pub(crate) fn output(file: &str, document: &yerba::Document, dry_run: bool) {
  if dry_run {
    println!("--- {}", file);
    print!("{}", document);
  } else {
    fs::write(file, document.to_string()).unwrap_or_else(|error| {
      eprintln!("Error writing {}: {}", file, error);
      process::exit(1);
    });
  }
}
