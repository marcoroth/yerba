mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_rename_key_same_level() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  document.rename("host", "hostname").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      hostname: localhost
      port: 5432
    "}
  );
}

#[test]
fn test_rename_nested_key_same_level() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document.rename("database.host", "database.hostname").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        hostname: localhost
        port: 5432
    "}
  );
}

#[test]
fn test_rename_preserves_comments() {
  let mut document = parse(indoc! {"
    # Config
    host: localhost
    # Port
    port: 5432
  "});

  document.rename("host", "hostname").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Config
      hostname: localhost
      # Port
      port: 5432
    "}
  );
}

#[test]
fn test_rename_preserves_quoted_key() {
  let mut document = parse("'host': localhost\n");

  document.rename("host", "hostname").unwrap();

  assert_eq!(document.to_string(), "'hostname': localhost\n");
}

#[test]
fn test_rename_nonexistent_key() {
  let mut document = parse("host: localhost\n");

  assert!(document.rename("missing", "new_name").is_err());
}

#[test]
fn test_rename_relocate_to_root() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document.rename("database.host", "hostname").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        port: 5432
      hostname: localhost
    "}
  );
}

#[test]
fn test_rename_relocate_to_other_map() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
    settings:
      debug: true
  "});

  document.rename("database.host", "settings.db_host").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        port: 5432
      settings:
        debug: true
        db_host: localhost
    "}
  );
}

#[test]
fn test_rename_relocate_from_root_to_nested() {
  let mut document = parse(indoc! {"
    hostname: localhost
    database:
      port: 5432
  "});

  document.rename("hostname", "database.host").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        port: 5432
        host: localhost
    "}
  );
}

#[test]
fn test_rename_with_bracket_index_path() {
  let mut document = parse(indoc! {"
    - id: first
      old_name: Alice
    - id: second
      old_name: Bob
  "});

  document.rename("[0].old_name", "[0].name").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        name: Alice
      - id: second
        old_name: Bob
    "}
  );
}
