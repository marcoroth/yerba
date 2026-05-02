mod json;

use std::fs;
use std::process;

use clap::{Parser, Subcommand};
use indoc::indoc;

#[derive(Parser)]
#[command(
  name = "yerba",
  version = yerba::version(),
  about = "Yerba 🧉 YAML Editing and Refactoring with Better Accuracy",
  arg_required_else_help = true,
  override_usage = "yerba [command] [options]",
  disable_help_subcommand = true,
  after_help = indoc! {r#"
    Examples:
      yerba get config.yml database.host
      yerba get videos.yml "[0].title"
      yerba set config.yml database.host 0.0.0.0
      yerba insert config.yml tags yaml --after ruby
      yerba insert speakers.yml "" --from speaker.yml --after ".name == Alice"
      yerba delete config.yml database.pool
      yerba find "data/**/videos.yml" "[]" --condition '.kind == keynote' --select 'id,title'
      yerba move videos.yml "" ".id == talk-2" --after ".id == talk-1"
      yerba sort-keys config.yml database 'id,host,port,name'
      yerba quote-style "data/**/*.yml" double
      yerba apply
  "#}
)]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  #[command(
    about = "Get values at a path (single or multi-value with [] brackets)",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba get config.yml database.host
        yerba get config.yml database.host --condition '.port == 5432'
        yerba get videos.yml "[].title"
        yerba get "data/**/videos.yml" "[].speakers[].name"
        yerba get videos.yml "[0].title"
    "#}
  )]
  Get {
    file: String,
    path: String,
    #[arg(long)]
    condition: Option<String>,
  },

  #[command(
    about = "Find and filter items with conditions, output as JSON or raw YAML",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba find "data/**/videos.yml" "[]" --condition '.kind == keynote'
        yerba find "data/**/videos.yml" "[]" --select 'id,title' --condition '.kind == keynote'
        yerba find "data/**/videos.yml" "[]" --select 'id,title,speakers[0].name'
        yerba find "data/**/videos.yml" "[]" --condition '.title contains Ruby' --raw
        yerba find "data/**/videos.yml" "[]" --condition '.speakers contains "Matz"'
    "#}
  )]
  Find {
    file: String,
    path: String,
    #[arg(long)]
    condition: Option<String>,
    /// Comma-separated fields to include (supports dot paths like speakers[].name)
    #[arg(long)]
    select: Option<String>,
    /// Output raw YAML instead of JSON
    #[arg(long)]
    raw: bool,
  },

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

  #[command(
    about = "Insert a new key into a map or item into a sequence",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba insert config.yml database.ssl true
        yerba insert config.yml database.ssl true --after host
        yerba insert config.yml database.ssl true --before port
        yerba insert config.yml tags yaml
        yerba insert config.yml tags yaml --at 0
        yerba insert config.yml tags yaml --after ruby
        yerba insert speakers.yml "" "name: Bob" --after ".name == Alice"
        yerba insert videos.yml "[0].speakers" "Diana" --before ".name == Charlie"
        yerba insert videos.yml "" --from new_talk.yml --after ".id == first-talk"
        cat talk.yml | yerba insert videos.yml "" --from - --after ".id == first-talk"
    "#}
  )]
  Insert {
    file: String,
    path: String,
    value: Option<String>,
    #[arg(long, help = "Read value from a file (use - for stdin)")]
    from: Option<String>,
    #[arg(long, help = "Insert before this value or condition (e.g. \".name == Alice\")")]
    before: Option<String>,
    #[arg(long, help = "Insert after this value or condition (e.g. \".name == Alice\")")]
    after: Option<String>,
    #[arg(long)]
    at: Option<usize>,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(
    about = "Rename a key in a map (preserves value and position)",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba rename config.yml database.host database.hostname
        yerba rename config.yml database.host hostname
        yerba rename config.yml database.host settings.db_host
        yerba rename videos.yml "[0].old_name" "[0].name"
    "#}
  )]
  Rename {
    file: String,
    source: String,
    destination: String,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(
    about = "Delete a key and its value from a map",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba delete config.yml database.pool
        yerba delete videos.yml "[0].description"
        yerba delete config.yml database.pool --dry-run
    "#}
  )]
  Delete {
    file: String,
    path: String,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(
    about = "Remove an item from a sequence by its value",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba remove config.yml tags rust
        yerba remove videos.yml "[0].speakers" Alice
    "#}
  )]
  Remove {
    file: String,
    path: String,
    value: String,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(
    about = "Move a sequence item to a new position",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba move config.yml tags rust --before ruby
        yerba move config.yml tags rust --after yaml
        yerba move config.yml tags 2 --to 0
        yerba move videos.yml "" ".id == talk-2" --after ".id == talk-1"
    "#}
  )]
  Move {
    file: String,
    path: String,
    #[arg(help = "Item to move: name, index, or condition (e.g. \".id == talk-1\")")]
    item: String,
    #[arg(long, help = "Move before this item or condition (e.g. \".id == talk-2\")")]
    before: Option<String>,
    #[arg(long, help = "Move after this item or condition (e.g. \".id == talk-2\")")]
    after: Option<String>,
    #[arg(long)]
    to: Option<usize>,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(
    about = "Move a key to a new position within a map",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba move-key config.yml database.pool --before database.host
        yerba move-key config.yml database.name --to 0
        yerba move-key config.yml database.pool --before ".host == localhost"
    "#}
  )]
  MoveKey {
    file: String,
    path: String,
    #[arg(long, help = "Move before this key or condition")]
    before: Option<String>,
    #[arg(long, help = "Move after this key or condition")]
    after: Option<String>,
    #[arg(long)]
    to: Option<usize>,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(
    about = "Sort keys in a map by a predefined order (aborts on unknown keys)",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba sort-keys config.yml database 'host,port,name,pool'
        yerba sort-keys "data/**/event.yml" "" "id,title,kind,location"
        yerba sort-keys "data/**/videos.yml" "[]" "id,title,speakers"
        yerba sort-keys config.yml database 'host,port' --dry-run
    "#}
  )]
  SortKeys {
    file: String,
    path: String,
    /// Comma-separated key order
    order: String,
    #[arg(long)]
    dry_run: bool,
  },

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
  Sort {
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
  },

  #[command(
    about = "Enforce a consistent quote style on values, keys, or both",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba quote-style config.yml double
        yerba quote-style config.yml plain --keys
        yerba quote-style config.yml double --all
        yerba quote-style config.yml single --path database.host
        yerba quote-style videos.yml plain --path "[].speakers"
    "#}
  )]
  QuoteStyle {
    file: String,
    /// Quote style
    style: yerba::QuoteStyle,
    /// Scope to a specific path
    #[arg(long)]
    path: Option<String>,
    /// Apply to keys only
    #[arg(long)]
    keys: bool,
    /// Apply to both keys and values
    #[arg(long)]
    all: bool,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(
    about = "Enforce blank lines between sequence entries",
    arg_required_else_help = true,
    after_help = indoc! {r#"
      Examples:
        yerba blank-lines videos.yml "" 1
        yerba blank-lines "data/**/videos.yml" "[]" 1
        yerba blank-lines videos.yml "[].speakers" 1
        yerba blank-lines config.yml tags 0
    "#}
  )]
  BlankLines {
    file: String,
    path: String,
    /// Number of blank lines between entries (0 = no blanks, 1 = one empty line)
    count: usize,
    #[arg(long)]
    dry_run: bool,
  },

  #[command(about = "Apply all rules from the Yerbafile and write changes")]
  Apply,
  #[command(about = "Check if all files match Yerbafile rules (exits 1 if not)")]
  Check,
  #[command(about = "Print the yerba version")]
  Version,
  #[command(about = "🧉")]
  Mate,
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Command::Get { file, path, condition } => {
      for resolved_file in resolve_files(&file) {
        let document = parse_file(&resolved_file);

        if let Some(condition) = &condition {
          let parent_path = path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

          if !document.evaluate_condition(parent_path, condition) {
            continue;
          }
        }

        let values = document.get_all(&path);

        if values.is_empty() && !path.contains('[') && !document.exists(&path) {
          eprintln!("Path not found: {}", path);
          process::exit(1);
        }

        for value in values {
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
      let select_fields: Option<Vec<&str>> = select.as_deref().map(|fields| fields.split(',').collect());

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

                result.insert("__file".to_string(), serde_json::Value::String(resolved_file.clone()));

                result.insert("__line".to_string(), serde_json::Value::Number(item.line.into()));

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
      let parent_path = path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

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
      from,
      before,
      after,
      at,
      dry_run,
    } => {
      let resolved_value = if let Some(from_path) = from {
        if from_path == "-" {
          use std::io::Read;
          let mut buffer = String::new();

          std::io::stdin().read_to_string(&mut buffer).unwrap_or_else(|error| {
            eprintln!("Error reading stdin: {}", error);

            std::process::exit(1);
          });

          buffer.trim().to_string()
        } else {
          std::fs::read_to_string(&from_path)
            .unwrap_or_else(|error| {
              eprintln!("Error reading {}: {}", from_path, error);
              std::process::exit(1);
            })
            .trim()
            .to_string()
        }
      } else if let Some(val) = value {
        val
      } else {
        eprintln!("Error: either a value argument or --from is required");

        std::process::exit(1);
      };

      let parent_path = path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

      let position = if let Some(index) = at {
        yerba::InsertPosition::At(index)
      } else if let Some(target) = before {
        if target.starts_with('.') {
          yerba::InsertPosition::BeforeCondition(target)
        } else {
          yerba::InsertPosition::Before(target)
        }
      } else if let Some(target) = after {
        if target.starts_with('.') {
          yerba::InsertPosition::AfterCondition(target)
        } else {
          yerba::InsertPosition::After(target)
        }
      } else {
        yerba::Yerbafile::find()
          .and_then(|yerbafile_path| yerba::Yerbafile::load(&yerbafile_path).ok())
          .and_then(|yerbafile| yerbafile.sort_order_for(&file, parent_path))
          .map(yerba::InsertPosition::FromSortOrder)
          .unwrap_or(yerba::InsertPosition::Last)
      };

      let mut document = parse_file(&file);
      run(|| document.insert_into(&path, &resolved_value, position));
      output(&file, &document, dry_run);
    }

    Command::Rename {
      file,
      source,
      destination,
      dry_run,
    } => {
      let mut document = parse_file(&file);
      run(|| document.rename(&source, &destination));
      output(&file, &document, dry_run);
    }

    Command::Delete { file, path, dry_run } => {
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
      before,
      after,
      to,
      dry_run,
    } => {
      let (parent_path, key) = path.rsplit_once('.').unwrap_or(("", &path));

      let mut document = parse_file(&file);

      let before_key = before.map(|target| {
        let (target_parent, target_key) = target.rsplit_once('.').unwrap_or(("", &target));

        if target_parent != parent_path {
          eprintln!(
            "Error: cannot move key across different maps ({} → {})\n\n  Use 'yerba rename' to relocate keys to a different path.",
            path, target
          );

          process::exit(1);
        }

        target_key.to_string()
      });

      let after_key = after.map(|target| {
        let (target_parent, target_key) = target.rsplit_once('.').unwrap_or(("", &target));

        if target_parent != parent_path {
          eprintln!(
            "Error: cannot move key across different maps ({} → {})\n\n  Use 'yerba rename' to relocate keys to a different path.",
            path, target
          );

          process::exit(1);
        }

        target_key.to_string()
      });

      let (from_index, to_index) = resolve_move_indexes(
        &document,
        parent_path,
        key,
        before_key,
        after_key,
        to,
        |document, parent_path, reference| document.resolve_key_index(parent_path, reference),
      );

      run(|| document.move_key(parent_path, from_index, to_index));
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
      keys,
      all,
      dry_run,
    } => {
      let dot_path = path.as_deref();

      for resolved_file in resolve_files(&file) {
        let mut document = parse_file(&resolved_file);

        if keys {
          let _ = document.enforce_key_style(&style, dot_path);
        } else if all {
          let _ = document.enforce_key_style(&style, dot_path);
          let _ = document.enforce_quotes_at(&style, dot_path);
        } else {
          let _ = document.enforce_quotes_at(&style, dot_path);
        }

        output(&resolved_file, &document, dry_run);
      }
    }

    Command::Sort {
      file,
      path,
      by,
      case_sensitive,
      dry_run,
    } => {
      let sort_fields = by.as_deref().map(yerba::SortField::parse_list).unwrap_or_default();

      for resolved_file in resolve_files(&file) {
        let mut document = parse_file(&resolved_file);

        if document.sort_items(&path, &sort_fields, case_sensitive).is_ok() {
          output(&resolved_file, &document, dry_run);
        }
      }
    }

    Command::BlankLines {
      file,
      path,
      count,
      dry_run,
    } => {
      for resolved_file in resolve_files(&file) {
        let mut document = parse_file(&resolved_file);

        if document.enforce_blank_lines(&path, count).is_ok() {
          output(&resolved_file, &document, dry_run);
        }
      }
    }

    Command::Apply => run_yerbafile(true),
    Command::Check => run_yerbafile(false),

    Command::Version => {
      println!("🧉 yerba v{}", yerba::version());
    }

    Command::Mate => {
      let g = "\x1b[32m";
      let b = "\x1b[1m";
      let d = "\x1b[2m";
      let i = "\x1b[3m";
      let y = "\x1b[33m";
      let r = "\x1b[0m";
      let hr = format!("    {d}──────────────────────────────────────────────────────────{r}");

      println!();
      println!("    {b}{g}🧉 YERBA MATE{r}");
      println!();
      println!("    {d}From the Guarani people of South America to the world.{r}");
      println!("    {d}Brewed from the leaves of{r} {i}Ilex paraguariensis{r}{d}, shared{r}");
      println!("    {d}in a hollowed calabaza, sipped through a metal bombilla.{r}");
      println!("    {d}One cup, many rounds.{r}");
      println!();
      println!("{hr}");
      println!();
      println!("    {b}{g}yerba{r} {d}/ˈʒɛɾ.ba/{r} {y}noun{r}");
      println!("    The dried leaves of {i}Ilex paraguariensis{r}, used to brew");
      println!("    mate. From Guarani {i}ka'a{r}, meaning \"herb\".");
      println!();
      println!("    {b}{g}mate{r} {d}/ˈma.te/{r} {y}noun{r}");
      println!("    A traditional South American caffeine-rich infusion.");
      println!("    Also the hollowed calabaza (gourd) from which it is drunk.");
      println!();
      println!("    {b}{g}bombilla{r} {d}/bomˈbi.ʒa/{r} {y}noun{r}");
      println!("    A metal straw with a filtered tip at the bottom, used");
      println!("    to sip mate without swallowing the leaves.");
      println!();
      println!("    {b}{g}cebador{r} {d}/se.baˈðoɾ/{r} {y}noun{r}");
      println!("    The person who prepares and serves mate to the group.");
      println!("    A role of care, not hierarchy.");
      println!();
      println!("    {b}{g}ronda{r} {d}/ˈron.da/{r} {y}noun{r}");
      println!("    The circle of people sharing mate. The cup passes");
      println!("    from hand to hand until it returns to the cebador.");
      println!();
      println!("    {b}{g}cebar{r} {d}/seˈbaɾ/{r} {y}verb{r}");
      println!("    To pour hot water over the yerba and serve a round.");
      println!("    The act of preparing each individual serving.");
      println!();
      println!("    {b}{g}aprontar{r} {d}/apɾonˈtaɾ/{r} {y}verb{r}");
      println!("    To set up mate before the first pour: arranging the");
      println!("    yerba, heating the water, positioning the bombilla.");
      println!();
      println!("    {b}{g}ensillar{r} {d}/ensiˈʒaɾ/{r} {y}verb{r}");
      println!("    To replace spent yerba with fresh leaves mid-session,");
      println!("    extending the life of the mate.");
      println!();
      println!("{hr}");
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
    match &error {
      yerba::YerbaError::IoError(io_error) => match io_error.kind() {
        std::io::ErrorKind::NotFound => eprintln!("Error: file not found: {}", file),
        std::io::ErrorKind::PermissionDenied => {
          eprintln!("Error: permission denied: {}", file)
        }
        _ => eprintln!("Error reading {}: {}", file, io_error),
      },
      _ => eprintln!("Error parsing {}: {}", file, error),
    }

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
