mod json;

use std::fs;
use std::process;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
  name = "yerba",
  version = yerba::version(),
  about = "Yerba 🧉 YAML Editing and Refactoring with Better Accuracy",
  override_usage = "yerba [command] [options]",
  disable_help_subcommand = true,
  after_help = "\x1b[1mExamples:\x1b[0m
  yerba get config.yml database.host
  yerba set config.yml database.host 0.0.0.0
  yerba insert config.yml database.ssl true --after host
  yerba delete config.yml database.pool
  yerba find \"data/**/videos.yml\" \"[]\" --condition '.kind == keynote' --select id,title
  yerba sort-keys config.yml database id,host,port,name
  yerba quote-style \"data/**/*.yml\" double
  yerba apply"
)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  Get {
    file: String,
    path: String,
    #[arg(long)]
    condition: Option<String>,
  },

  GetAll {
    file: String,
    path: String,
    #[arg(long)]
    condition: Option<String>,
  },

  Find {
    file: String,
    path: String,
    #[arg(long)]
    condition: Option<String>,
    #[arg(long)]
    select: Option<String>,
    #[arg(long)]
    raw: bool,
  },

  Set {
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
  },

  Insert {
    file: String,
    path: String,
    value: String,
    #[arg(long)]
    before: Option<String>,
    #[arg(long)]
    after: Option<String>,
    #[arg(long)]
    at: Option<usize>,
    #[arg(long)]
    dry_run: bool,
  },

  Rename {
    file: String,
    path: String,
    new_key: String,
    #[arg(long)]
    dry_run: bool,
  },

  Delete {
    file: String,
    path: String,
    #[arg(long)]
    dry_run: bool,
  },

  Remove {
    file: String,
    path: String,
    value: String,
    #[arg(long)]
    dry_run: bool,
  },

  Move {
    file: String,
    path: String,
    item: String,
    #[arg(long)]
    before: Option<String>,
    #[arg(long)]
    after: Option<String>,
    #[arg(long)]
    to: Option<usize>,
    #[arg(long)]
    dry_run: bool,
  },

  MoveKey {
    file: String,
    path: String,
    key: String,
    #[arg(long)]
    before: Option<String>,
    #[arg(long)]
    after: Option<String>,
    #[arg(long)]
    to: Option<usize>,
    #[arg(long)]
    dry_run: bool,
  },

  SortKeys {
    file: String,
    path: String,
    order: String,
    #[arg(long)]
    dry_run: bool,
  },

  QuoteStyle {
    file: String,
    style: String,
    #[arg(long)]
    path: Option<String>,
    #[arg(long)]
    dry_run: bool,
  },

  Apply,
  Check,
  Version,
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Command::Get {
      file,
      path,
      condition,
    } => {
      let document = parse_file(&file);

      if let Some(condition) = &condition {
        let parent_path = path
          .rsplit_once('.')
          .map(|(parent, _)| parent)
          .unwrap_or("");

        if !document.evaluate_condition(parent_path, condition) {
          process::exit(0);
        }
      }

      match document.get(&path) {
        Some(value) => println!("{}", value),
        None => {
          eprintln!("Path not found: {}", path);
          process::exit(1);
        }
      }
    }

    Command::GetAll {
      file,
      path,
      condition,
    } => {
      for resolved_file in resolve_files(&file) {
        let document = parse_file(&resolved_file);

        if let Some(condition) = &condition {
          let parent_path = path
            .rsplit_once('.')
            .map(|(parent, _)| parent)
            .unwrap_or("");

          if !document.evaluate_condition(parent_path, condition) {
            continue;
          }
        }

        for value in document.get_all(&path) {
          println!("{}", value);
        }
      }
    }

    Command::Find {
      file,
      path,
      condition,
      select,
      raw,
    } => {
      let select_fields: Option<Vec<&str>> =
        select.as_deref().map(|fields| fields.split(',').collect());

      if raw {
        for resolved_file in resolve_files(&file) {
          let document = parse_file(&resolved_file);

          let matches = match &condition {
            Some(condition) => document.find_items(&path, condition),
            None => document.find_all(&path),
          };

          for (index, item) in matches.iter().enumerate() {
            if index > 0 {
              println!();
            }

            eprintln!("# {}:{}", resolved_file, item.line);
            println!("{}", item.text);
          }
        }
      } else {
        let mut all_results: Vec<serde_json::Value> = Vec::new();

        for resolved_file in resolve_files(&file) {
          let document = parse_file(&resolved_file);

          let matches = match &condition {
            Some(condition) => document.find_items(&path, condition),
            None => document.find_all(&path),
          };

          for item in &matches {
            let yaml_with_dash = format!("- {}", item.text.trim_start_matches("- "));

            if let Ok(parsed) = serde_yaml::from_str::<Vec<serde_yaml::Value>>(&yaml_with_dash) {
              for value in parsed {
                let mut result = serde_json::Map::new();

                result.insert(
                  "__file".to_string(),
                  serde_json::Value::String(resolved_file.clone()),
                );

                result.insert(
                  "__line".to_string(),
                  serde_json::Value::Number(item.line.into()),
                );

                match &select_fields {
                  Some(fields) => {
                    for field in fields {
                      let json_value = json::resolve_select_field(&value, field);
                      let json_key = json::select_field_key(field);

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

                        result.insert(json_key, json::yaml_to_json(yaml_value));
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

    Command::Set {
      file,
      path,
      value,
      if_exists,
      if_missing,
      condition,
      dry_run,
    } => {
      let mut document = parse_file(&file);
      let parent_path = path
        .rsplit_once('.')
        .map(|(parent, _)| parent)
        .unwrap_or("");

      let should_set = if if_exists {
        document.exists(&path)
      } else if if_missing {
        !document.exists(&path)
      } else if let Some(condition) = &condition {
        document.evaluate_condition(parent_path, condition)
      } else {
        true
      };

      if should_set {
        run(|| document.set(&path, &value));
      }

      output(&file, &document, dry_run);
    }

    Command::Insert {
      file,
      path,
      value,
      before,
      after,
      at,
      dry_run,
    } => {
      let parent_path = path
        .rsplit_once('.')
        .map(|(parent, _)| parent)
        .unwrap_or("");

      let position = if let Some(index) = at {
        yerba::InsertPosition::At(index)
      } else if let Some(target) = before {
        yerba::InsertPosition::Before(target)
      } else if let Some(target) = after {
        yerba::InsertPosition::After(target)
      } else {
        yerba::Yerbafile::find()
          .and_then(|yerbafile_path| yerba::Yerbafile::load(&yerbafile_path).ok())
          .and_then(|yerbafile| yerbafile.sort_order_for(&file, parent_path))
          .map(yerba::InsertPosition::FromSortOrder)
          .unwrap_or(yerba::InsertPosition::Last)
      };

      let mut document = parse_file(&file);
      run(|| document.insert_into(&path, &value, position));
      output(&file, &document, dry_run);
    }

    Command::Rename {
      file,
      path,
      new_key,
      dry_run,
    } => {
      let mut document = parse_file(&file);
      run(|| document.rename(&path, &new_key));
      output(&file, &document, dry_run);
    }

    Command::Delete {
      file,
      path,
      dry_run,
    } => {
      let mut document = parse_file(&file);
      run(|| document.delete(&path));
      output(&file, &document, dry_run);
    }

    Command::Remove {
      file,
      path,
      value,
      dry_run,
    } => {
      let mut document = parse_file(&file);
      run(|| document.remove(&path, &value));
      output(&file, &document, dry_run);
    }

    Command::Move {
      file,
      path,
      item,
      before,
      after,
      to,
      dry_run,
    } => {
      let mut document = parse_file(&file);
      let (from_index, to_index) = resolve_move_indexes(
        &document,
        &path,
        &item,
        before,
        after,
        to,
        |document, path, reference| document.resolve_sequence_index(path, reference),
      );

      run(|| document.move_item(&path, from_index, to_index));
      output(&file, &document, dry_run);
    }

    Command::MoveKey {
      file,
      path,
      key,
      before,
      after,
      to,
      dry_run,
    } => {
      let mut document = parse_file(&file);
      let (from_index, to_index) = resolve_move_indexes(
        &document,
        &path,
        &key,
        before,
        after,
        to,
        |document, path, reference| document.resolve_key_index(path, reference),
      );

      run(|| document.move_key(&path, from_index, to_index));
      output(&file, &document, dry_run);
    }

    Command::SortKeys {
      file,
      path,
      order,
      dry_run,
    } => {
      let key_order: Vec<&str> = order.split(',').collect();
      let files = resolve_files(&file);

      let mut has_errors = false;

      for resolved_file in &files {
        let document = parse_file(resolved_file);

        if let Err(error) = document.validate_sort_keys(&path, &key_order) {
          eprintln!("Error in {}: {}", resolved_file, error);
          has_errors = true;
        }
      }

      if has_errors {
        process::exit(1);
      }

      for resolved_file in &files {
        let mut document = parse_file(resolved_file);

        if document.sort_keys(&path, &key_order).is_ok() {
          output(resolved_file, &document, dry_run);
        }
      }
    }

    Command::QuoteStyle {
      file,
      style,
      path,
      dry_run,
    } => {
      let parsed_style: yerba::QuoteStyle = style.parse().unwrap_or_else(|error| {
        eprintln!("{}", error);
        process::exit(1);
      });

      let dot_path = path.as_deref();

      for resolved_file in resolve_files(&file) {
        let mut document = parse_file(&resolved_file);

        if document.enforce_quotes_at(&parsed_style, dot_path).is_ok() {
          output(&resolved_file, &document, dry_run);
        }
      }
    }

    Command::Apply => run_yerbafile(true),
    Command::Check => run_yerbafile(false),

    Command::Version => {
      println!("🧉 yerba v{}", yerba::version());
    }
  }
}

fn run_yerbafile(write: bool) {
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

fn resolve_move_indexes(
  document: &yerba::Document,
  path: &str,
  item: &str,
  before: Option<String>,
  after: Option<String>,
  to: Option<usize>,
  resolve: impl Fn(&yerba::Document, &str, &str) -> Result<usize, yerba::YerbaError>,
) -> (usize, usize) {
  let from_index = resolve(document, path, item).unwrap_or_else(|error| {
    eprintln!("Error: {}", error);
    process::exit(1);
  });

  let to_index = if let Some(index) = to {
    index
  } else if let Some(target) = &before {
    let target_index = resolve(document, path, target).unwrap_or_else(|error| {
      eprintln!("Error: {}", error);
      process::exit(1);
    });

    if from_index < target_index {
      target_index - 1
    } else {
      target_index
    }
  } else if let Some(target) = &after {
    let target_index = resolve(document, path, target).unwrap_or_else(|error| {
      eprintln!("Error: {}", error);
      process::exit(1);
    });

    if from_index <= target_index {
      target_index
    } else {
      target_index + 1
    }
  } else {
    eprintln!("Error: specify --before, --after, or --to");
    process::exit(1);
  };

  (from_index, to_index)
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
