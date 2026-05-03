use std::path::Path;
use std::process;

use indoc::indoc;

use super::color::*;

pub fn run() {
  let filename = "Yerbafile";

  if Path::new(filename).exists() {
    eprintln!("🧉 {YELLOW}{BOLD}Yerbafile already exists.{RESET}");
    process::exit(1);
  }

  let content = indoc! {r#"
    # rules:
    #   - files: "**/*.yml"
    #     pipeline:
    #
    #       # Enforce quote style on values and keys
    #       - quote_style:
    #           key_style: plain        # plain, single, double
    #           value_style: double     # plain, single, double
    #
    #       # Override quote style for a specific selector
    #       - quote_style:
    #           path: "[].speakers"
    #           value_style: plain
    #
    #       # Sort keys in a predefined order (aborts on unknown keys)
    #       - sort_keys:
    #           path: "[]"
    #           order:
    #             - id
    #             - title
    #             - description
    #
    #       # Sort items in a sequence by field(s)
    #       - sort:
    #           path: ""
    #           by: name
    #           # case_sensitive: false
    #
    #       # Enforce blank lines between sequence entries
    #       - blank_lines:
    #           count: 1
    #
    #       # Set a value (with optional condition)
    #       - set:
    #           path: "status"
    #           value: "published"
    #           # condition: ".draft == true"
    #
    #       # Insert a new key or sequence item
    #       - insert:
    #           path: "tags"
    #           value: "yaml"
    #           # condition: ".tags not_contains yaml"
    #
    #       # Delete a key (with optional condition)
    #       - delete:
    #           path: "deprecated_field"
    #           # condition: ".deprecated_field == true"
    #
    #       # Rename a key
    #       - rename:
    #           from: "old_name"
    #           to: "new_name"
    #
    #       # Remove an item from a sequence
    #       - remove:
    #           path: "tags"
    #           value: "obsolete"
    #
    #       # Get a value and store it as a variable
    #       - get:
    #           path: "[].name"
    #           as: "names"
    #           # file: "other_file.yml"
  "#};

  std::fs::write(filename, content).unwrap_or_else(|error| {
    eprintln!("{RED}Error:{RESET} writing Yerbafile: {}", error);
    process::exit(1);
  });

  eprintln!("🧉 {GREEN}{BOLD}Created Yerbafile{RESET}");
}
