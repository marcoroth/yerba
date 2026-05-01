use std::env;
use std::fs;
use std::process;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    println!("🧉 yerba v{}", yerba::version());
    println!();
    println!("Usage:");
    println!("  yerba get <file> <path> [--condition <cond>]");
    println!("  yerba get-all <file> <path> [--condition <cond>]");
    println!("  yerba find <file> <path> [--select <fields>] [--condition <cond>] [--raw]");
    println!(
      "  yerba set <file> <path> <value> [--if-exists] [--if-missing] [--condition <cond>] [--dry-run]"
    );
    println!(
      "  yerba insert <file> <path.key> <value> [--before|--after <key>] [--at <index>] [--dry-run]"
    );
    println!("  yerba rename <file> <path> <new_key> [--dry-run]");
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
        eprintln!("Usage: yerba get <file> <path> [--condition <cond>]");
        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let condition = parse_option(&args, "--condition");

      let document = parse_file(file);

      if let Some(condition) = condition {
        let parent_path = path
          .rsplit_once('.')
          .map(|(parent, _)| parent)
          .unwrap_or("");

        if !document.evaluate_condition(parent_path, condition) {
          process::exit(0);
        }
      }

      match document.get(path) {
        Some(value) => println!("{}", value),
        None => {
          eprintln!("Path not found: {}", path);
          process::exit(1);
        }
      }
    }

    "get-all" => {
      if args.len() < 4 {
        eprintln!("Usage: yerba get-all <file> <path> [--condition <cond>]");
        process::exit(1);
      }

      let file_pattern = &args[2];
      let path = &args[3];
      let condition = parse_option(&args, "--condition");

      for file in resolve_files(file_pattern) {
        let document = parse_file(&file);

        if let Some(condition) = condition {
          let parent_path = path
            .rsplit_once('.')
            .map(|(parent, _)| parent)
            .unwrap_or("");

          if !document.evaluate_condition(parent_path, condition) {
            continue;
          }
        }

        for value in document.get_all(path) {
          println!("{}", value);
        }
      }
    }

    "find" => {
      if args.len() < 4 {
        eprintln!(
          "Usage: yerba find <file> <path> [--select <fields>] [--condition <cond>] [--raw]"
        );

        process::exit(1);
      }

      let file_pattern = &args[2];
      let path = &args[3];
      let raw = args.iter().any(|arg| arg == "--raw");
      let condition = parse_option(&args, "--condition");
      let select_fields: Option<Vec<&str>> =
        parse_option(&args, "--select").map(|fields| fields.split(',').collect());

      if raw {
        for file in resolve_files(file_pattern) {
          let document = parse_file(&file);

          let matches = match condition {
            Some(condition) => document.find_items(path, condition),
            None => document.find_all(path),
          };

          for (index, item) in matches.iter().enumerate() {
            if index > 0 {
              println!();
            }

            eprintln!("# {}:{}", file, item.line);
            println!("{}", item.text);
          }
        }
      } else {
        let mut all_results: Vec<serde_json::Value> = Vec::new();

        for file in resolve_files(file_pattern) {
          let document = parse_file(&file);

          let matches = match condition {
            Some(condition) => document.find_items(path, condition),
            None => document.find_all(path),
          };

          for item in &matches {
            let yaml_with_dash = format!("- {}", item.text.trim_start_matches("- "));

            if let Ok(parsed) = serde_yaml::from_str::<Vec<serde_yaml::Value>>(&yaml_with_dash) {
              for value in parsed {
                let mut result = serde_json::Map::new();

                result.insert(
                  "__file".to_string(),
                  serde_json::Value::String(file.clone()),
                );

                result.insert(
                  "__line".to_string(),
                  serde_json::Value::Number(item.line.into()),
                );

                match &select_fields {
                  Some(fields) => {
                    for field in fields {
                      let json_value = resolve_select_field(&value, field);
                      let json_key = select_field_key(field);

                      result.insert(json_key, json_value);
                    }
                  }

                  None => {
                    if let serde_yaml::Value::Mapping(map) = &value {
                      for (key, yaml_value) in map {
                        let json_key = match key {
                          serde_yaml::Value::String(string) => string.clone(),
                          _ => format!("{:?}", key),
                        };

                        result.insert(json_key, yaml_to_json(yaml_value));
                      }
                    }
                  }
                }

                all_results.push(serde_json::Value::Object(result));
              }
            }
          }
        }

        println!(
          "{}",
          serde_json::to_string_pretty(&all_results).unwrap_or_else(|_| "[]".to_string())
        );
      }
    }

    "set" => {
      if args.len() < 5 {
        eprintln!(
          "Usage: yerba set <file> <path> <value> [--if-exists] [--if-missing] [--condition <cond>] [--dry-run]"
        );

        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let value = &args[4];

      let if_exists = args.iter().any(|arg| arg == "--if-exists");
      let if_missing = args.iter().any(|arg| arg == "--if-missing");
      let if_condition = parse_option(&args, "--condition");

      let mut document = parse_file(file);

      let parent_path = path
        .rsplit_once('.')
        .map(|(parent, _)| parent)
        .unwrap_or("");

      let should_set = if if_exists {
        document.exists(path)
      } else if if_missing {
        !document.exists(path)
      } else if let Some(condition) = if_condition {
        document.evaluate_condition(parent_path, condition)
      } else {
        true
      };

      if should_set {
        run(|| document.set(path, value));
      }

      output(file, &document, dry_run);
    }

    "insert" => {
      if args.len() < 5 {
        eprintln!(
          "Usage: yerba insert <file> <path.key> <value> [--before|--after <key>] [--at <index>] [--dry-run]"
        );

        process::exit(1);
      }

      let file = &args[2];
      let path = &args[3];
      let value = &args[4];

      let parent_path = path
        .rsplit_once('.')
        .map(|(parent, _)| parent)
        .unwrap_or("");

      let position = if let Some(index) = parse_option(&args, "--at") {
        yerba::InsertPosition::At(index.parse::<usize>().unwrap_or_else(|_| {
          eprintln!("Error: --at requires a numeric index");
          process::exit(1);
        }))
      } else if let Some(target) = parse_option(&args, "--before") {
        yerba::InsertPosition::Before(target.to_string())
      } else if let Some(target) = parse_option(&args, "--after") {
        yerba::InsertPosition::After(target.to_string())
      } else {
        yerba::Yerbafile::find()
          .and_then(|yerbafile_path| yerba::Yerbafile::load(&yerbafile_path).ok())
          .and_then(|yerbafile| yerbafile.sort_order_for(file, parent_path))
          .map(yerba::InsertPosition::FromSortOrder)
          .unwrap_or(yerba::InsertPosition::Last)
      };

      let mut document = parse_file(file);
      run(|| document.insert_into(path, value, position));
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

      let style: yerba::QuoteStyle = style_name.parse().unwrap_or_else(|error| {
        eprintln!("{}", error);

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
      let files = resolve_files(file_pattern);

      let mut has_errors = false;

      for file in &files {
        let document = parse_file(file);

        if let Err(error) = document.validate_sort_keys(path, &key_order) {
          eprintln!("Error in {}: {}", file, error);
          has_errors = true;
        }
      }

      if has_errors {
        process::exit(1);
      }

      for file in &files {
        let mut document = parse_file(file);

        if document.sort_keys(path, &key_order).is_ok() {
          output(file, &document, dry_run);
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

fn parse_option<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
  args
    .iter()
    .position(|arg| arg == flag)
    .and_then(|index| args.get(index + 1))
    .map(|value| value.as_str())
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
    print!("{}", document);
  } else {
    fs::write(file, document.to_string()).unwrap_or_else(|error| {
      eprintln!("Error writing {}: {}", file, error);
      process::exit(1);
    });
  }
}

fn yaml_to_json(value: &serde_yaml::Value) -> serde_json::Value {
  match value {
    serde_yaml::Value::Null => serde_json::Value::Null,
    serde_yaml::Value::Bool(boolean) => serde_json::Value::Bool(*boolean),

    serde_yaml::Value::Number(number) => {
      if let Some(integer) = number.as_i64() {
        serde_json::Value::Number(integer.into())
      } else if let Some(float) = number.as_f64() {
        serde_json::json!(float)
      } else {
        serde_json::Value::String(number.to_string())
      }
    }

    serde_yaml::Value::String(string) => serde_json::Value::String(string.clone()),

    serde_yaml::Value::Sequence(sequence) => {
      serde_json::Value::Array(sequence.iter().map(yaml_to_json).collect())
    }

    serde_yaml::Value::Mapping(mapping) => {
      let mut map = serde_json::Map::new();

      for (key, yaml_value) in mapping {
        let json_key = match key {
          serde_yaml::Value::String(string) => string.clone(),
          _ => format!("{:?}", key),
        };

        map.insert(json_key, yaml_to_json(yaml_value));
      }

      serde_json::Value::Object(map)
    }

    serde_yaml::Value::Tagged(tagged) => yaml_to_json(&tagged.value),
  }
}

fn resolve_select_field(value: &serde_yaml::Value, field: &str) -> serde_json::Value {
  if !field.contains('.') && !field.contains('[') {
    if let serde_yaml::Value::Mapping(map) = value {
      for (key, yaml_value) in map {
        if let serde_yaml::Value::String(key_string) = key {
          if key_string == field {
            return yaml_to_json(yaml_value);
          }
        }
      }
    }

    return serde_json::Value::Null;
  }

  let mut current_values = vec![value.clone()];

  for segment in parse_select_segments(field) {
    let mut next_values = Vec::new();

    for current in &current_values {
      if segment.starts_with('[') {
        if let serde_yaml::Value::Sequence(sequence) = current {
          let index = segment
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
            .and_then(|inner| {
              if inner.is_empty() {
                None
              } else {
                inner.parse::<usize>().ok()
              }
            });

          match index {
            None => next_values.extend(sequence.iter().cloned()),
            Some(index) => {
              if let Some(item) = sequence.get(index) {
                next_values.push(item.clone());
              }
            }
          }
        }
      } else if let serde_yaml::Value::Mapping(map) = current {
        for (map_key, yaml_value) in map {
          if let serde_yaml::Value::String(key_string) = map_key {
            if key_string == segment {
              next_values.push(yaml_value.clone());
            }
          }
        }
      }
    }

    current_values = next_values;
  }

  let used_brackets = field.contains("[]");

  if current_values.is_empty() {
    if used_brackets {
      serde_json::Value::Array(Vec::new())
    } else {
      serde_json::Value::Null
    }
  } else if current_values.len() == 1 && !used_brackets {
    yaml_to_json(&current_values[0])
  } else {
    serde_json::Value::Array(current_values.iter().map(yaml_to_json).collect())
  }
}

fn select_field_key(field: &str) -> String {
  let root = field.split(['.', '[']).next().unwrap_or(field);

  root.to_string()
}

fn parse_select_segments(field: &str) -> Vec<&str> {
  let mut segments = Vec::new();
  let mut rest = field;

  while !rest.is_empty() {
    if rest.starts_with('[') {
      if let Some(close) = rest.find(']') {
        segments.push(&rest[..close + 1]);
        rest = &rest[close + 1..];

        if rest.starts_with('.') {
          rest = &rest[1..];
        }
      } else {
        segments.push(rest);
        break;
      }
    } else {
      let dot_index = rest.find('.');
      let bracket_index = rest.find('[');

      let split_at = match (dot_index, bracket_index) {
        (Some(dot), Some(bracket)) => Some(dot.min(bracket)),
        (Some(dot), None) => Some(dot),
        (None, Some(bracket)) => Some(bracket),
        (None, None) => None,
      };

      match split_at {
        Some(index) => {
          let segment = &rest[..index];

          if !segment.is_empty() {
            segments.push(segment);
          }

          rest = &rest[index..];

          if rest.starts_with('.') {
            rest = &rest[1..];
          }
        }

        None => {
          segments.push(rest);
          break;
        }
      }
    }
  }

  segments
}
