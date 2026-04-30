use std::env;
use std::fs;
use std::process;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    println!("🧉 yerba v{}", yerba::version());
    println!();
    println!("Usage:");
    println!("  yerba get <file> <path>");
    println!("  yerba set <file> <path> <value> [--dry-run]");
    println!("  yerba rename <file> <path> <new_key> [--dry-run]");
    println!("  yerba append <file> <path> <value> [--dry-run]");
    println!("  yerba delete <file> <path> [--dry-run]");
    println!("  yerba remove <file> <path> <value> [--dry-run]");
    println!("  yerba move <file> <path> <item> --before <target> [--dry-run]");
    println!("  yerba move <file> <path> <item> --after <target> [--dry-run]");
    println!("  yerba move <file> <path> <item> --to <index> [--dry-run]");
    println!("  yerba move-key <file> <path> <key> --before <target> [--dry-run]");
    println!("  yerba move-key <file> <path> <key> --after <target> [--dry-run]");
    println!("  yerba move-key <file> <path> <key> --to <index> [--dry-run]");
    println!("  yerba sort-keys <file> <path> <key1,key2,...> [--dry-run]");
    println!("  yerba quote-style <file> <style> [--path <dot.path>] [--dry-run]");
    println!();
    println!("Quote styles: plain, single, double");
    println!();
    println!("Yerbafile:");
    println!("  yerba apply            Apply all rules from Yerbafile");
    println!("  yerba check            Check if files match Yerbafile rules (exit 1 if not)");
    println!();
    println!("Options:");
    println!("  --dry-run  Print result to stdout instead of writing to file");
    println!();
    println!("Items and targets can be names or indexes (e.g. 'ruby' or '0')");

    process::exit(0);
  }

  let dry_run = args.iter().any(|arg| arg == "--dry-run");
  let command = &args[1];

  match command.as_str() {
    "get" => {
      if args.len() < 4 {
        eprintln!("Usage: yerba get <file> <path>");
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];

      let document = parse_file(file);

      match document.get(path) {
        Some(value) => println!("{}", value),
        None => {
          eprintln!("Path not found: {}", path);
          process::exit(1);
        }
      }
    }

    "set" => {
      if args.len() < 5 {
        eprintln!("Usage: yerba set <file> <path> <value> [--dry-run]");
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let value = &args[4];

      let mut document = parse_file(file);
      run(|| document.set(path, value));
      output(file, &document, dry_run);
    }

    "rename" => {
      if args.len() < 5 {
        eprintln!("Usage: yerba rename <file> <path> <new_key> [--dry-run]");
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let new_key = &args[4];

      let mut document = parse_file(file);
      run(|| document.rename(path, new_key));
      output(file, &document, dry_run);
    }

    "append" => {
      if args.len() < 5 {
        eprintln!("Usage: yerba append <file> <path> <value> [--dry-run]");
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let value = &args[4];

      let mut document = parse_file(file);
      run(|| document.append(path, value));
      output(file, &document, dry_run);
    }

    "delete" => {
      if args.len() < 4 {
        eprintln!("Usage: yerba delete <file> <path> [--dry-run]");
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];

      let mut document = parse_file(file);
      run(|| document.delete(path));
      output(file, &document, dry_run);
    }

    "remove" => {
      if args.len() < 5 {
        eprintln!("Usage: yerba remove <file> <path> <value> [--dry-run]");
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let value = &args[4];

      let mut document = parse_file(file);
      run(|| document.remove(path, value));
      output(file, &document, dry_run);
    }

    "apply" | "check" => {
      let write = command == "apply";
      let yerbafile_path = yerba::Yerbafile::find().unwrap_or_else(|| {
        eprintln!("No Yerbafile found. Create one in the current directory or a parent.");
        process::exit(1);
      });

      let yerbafile = yerba::Yerbafile::load(&yerbafile_path).unwrap_or_else(|error| {
        eprintln!("Error loading {}: {}", yerbafile_path.display(), error);
        process::exit(1);
      });

      eprintln!("🧉 Using {}", yerbafile_path.display());

      let results = yerbafile.apply(write);
      let mut has_changes = false;
      let mut has_errors = false;

      for result in &results {
        if let Some(error) = &result.error {
          eprintln!("  error: {} — {}", result.file, error);
          has_errors = true;
        } else if result.changed {
          if write {
            eprintln!("  updated: {}", result.file);
          } else {
            eprintln!("  would change: {}", result.file);
          }
          has_changes = true;
        }
      }

      if !has_changes && !has_errors {
        eprintln!("  All files match the rules.");
      }

      if !write && has_changes {
        process::exit(1);
      }

      if has_errors {
        process::exit(1);
      }
    }

    "version" => {
      println!("🧉 yerba v{}", yerba::version());
    }

    "quote-style" => {
      if args.len() < 4 {
        eprintln!("Usage: yerba quote-style <file> <style> [--path <dot.path>] [--dry-run]");
        eprintln!("Styles: plain, single, double");
        process::exit(1);
      }

      let file_pattern = &args[2];
      let style_name = &args[3];

      let style = yerba::QuoteStyle::from_str(style_name).unwrap_or_else(|| {
        eprintln!(
          "Unknown quote style: '{}'. Use: plain, single, double",
          style_name
        );

        process::exit(1);
      });

      let dot_path = args
        .iter()
        .position(|arg| arg == "--path")
        .and_then(|index| args.get(index + 1))
        .map(|path| path.as_str());

      for file in resolve_files(file_pattern) {
        let mut document = parse_file(&file);

        if document.enforce_quotes_at(&style, dot_path).is_ok() {
          output(&file, &document, dry_run);
        }
      }
    }

    "sort-keys" => {
      if args.len() < 5 {
        eprintln!("Usage: yerba sort-keys <file> <path> <key1,key2,...> [--dry-run]");
        process::exit(1);
      }

      let file_pattern = &args[2];
      let path = &args[3];
      let key_order: Vec<&str> = args[4].split(',').collect();

      for file in resolve_files(file_pattern) {
        let mut document = parse_file(&file);
        if document.sort_keys(path, &key_order).is_ok() {
          output(&file, &document, dry_run);
        }
      }
    }

    "move-key" => {
      if args.len() < 6 {
        eprintln!(
          "Usage: yerba move-key <file> <path> <key> --before|--after|--to <target> [--dry-run]"
        );

        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let key = &args[4];
      let direction = &args[5];

      let target = args.get(6).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("Usage: yerba move-key <file> <path> <key> --before|--after|--to <target>");
        process::exit(1);
      });

      let mut document = parse_file(file);

      let from_index = document
        .resolve_key_index(path, key)
        .unwrap_or_else(|error| {
          eprintln!("Error: {}", error);
          process::exit(1);
        });

      let to_index = match direction.as_str() {
        "--to" => target.parse::<usize>().unwrap_or_else(|_| {
          eprintln!("Error: --to requires a numeric index");

          process::exit(1);
        }),

        "--before" => {
          let target_index = document
            .resolve_key_index(path, target)
            .unwrap_or_else(|error| {
              eprintln!("Error: {}", error);
              process::exit(1);
            });

          if from_index < target_index {
            target_index - 1
          } else {
            target_index
          }
        }

        "--after" => {
          let target_index = document
            .resolve_key_index(path, target)
            .unwrap_or_else(|error| {
              eprintln!("Error: {}", error);
              process::exit(1);
            });

          if from_index <= target_index {
            target_index
          } else {
            target_index + 1
          }
        }

        _ => {
          eprintln!(
            "Unknown direction: {}. Use --before, --after, or --to",
            direction
          );

          process::exit(1);
        }
      };

      run(|| document.move_key(path, from_index, to_index));
      output(file, &document, dry_run);
    }

    "move" => {
      if args.len() < 6 {
        eprintln!(
          "Usage: yerba move <file> <path> <item> --before|--after|--to <target> [--dry-run]"
        );
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let item = &args[4];
      let direction = &args[5];

      let target = args.get(6).map(|s| s.as_str()).unwrap_or_else(|| {
        eprintln!("Usage: yerba move <file> <path> <item> --before|--after|--to <target>");
        process::exit(1);
      });

      let mut document = parse_file(file);

      let from_index = document
        .resolve_sequence_index(path, item)
        .unwrap_or_else(|error| {
          eprintln!("Error: {}", error);
          process::exit(1);
        });

      let to_index = match direction.as_str() {
        "--to" => target.parse::<usize>().unwrap_or_else(|_| {
          eprintln!("Error: --to requires a numeric index");
          process::exit(1);
        }),

        "--before" => {
          let target_index = document
            .resolve_sequence_index(path, target)
            .unwrap_or_else(|error| {
              eprintln!("Error: {}", error);
              process::exit(1);
            });

          if from_index < target_index {
            target_index - 1
          } else {
            target_index
          }
        }

        "--after" => {
          let target_index = document
            .resolve_sequence_index(path, target)
            .unwrap_or_else(|error| {
              eprintln!("Error: {}", error);
              process::exit(1);
            });

          if from_index <= target_index {
            target_index
          } else {
            target_index + 1
          }
        }

        _ => {
          eprintln!(
            "Unknown direction: {}. Use --before, --after, or --to",
            direction
          );
          process::exit(1);
        }
      };

      run(|| document.move_item(path, from_index, to_index));
      output(file, &document, dry_run);
    }

    _ => {
      eprintln!("Unknown command: {}", command);
      process::exit(1);
    }
  }
}

fn resolve_files(pattern: &str) -> Vec<String> {
  if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
    let paths: Vec<String> = glob::glob(pattern)
      .unwrap_or_else(|error| {
        eprintln!("Invalid glob pattern '{}': {}", pattern, error);
        process::exit(1);
      })
      .filter_map(|entry| entry.ok())
      .map(|path| path.to_string_lossy().to_string())
      .collect();

    if paths.is_empty() {
      eprintln!("No files matched pattern: {}", pattern);
      process::exit(1);
    }

    paths
  } else {
    vec![pattern.to_string()]
  }
}

fn parse_file(file: &str) -> yerba::Document {
  yerba::parse_file(file).unwrap_or_else(|error| {
    eprintln!("Error parsing {}: {}", file, error);
    process::exit(1);
  })
}

fn run(operation: impl FnOnce() -> Result<(), yerba::YerbaError>) {
  operation().unwrap_or_else(|error| {
    eprintln!("Error: {}", error);
    process::exit(1);
  });
}

fn output(file: &str, document: &yerba::Document, dry_run: bool) {
  if dry_run {
    println!("--- {}", file);
    print!("{}", document.to_string());
  } else {
    fs::write(file, document.to_string()).unwrap_or_else(|error| {
      eprintln!("Error writing {}: {}", file, error);
      process::exit(1);
    });
  }
}
