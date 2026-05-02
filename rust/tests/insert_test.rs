mod support;
use indoc::indoc;
use support::parse;
use yerba::InsertPosition;

#[test]
fn test_insert_key_at_end() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document
    .insert_into("database.ssl", "true", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        ssl: true
    "}
  );
}

#[test]
fn test_insert_key_at_index() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document
    .insert_into("database.ssl", "true", InsertPosition::At(0))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        ssl: true
        host: localhost
        port: 5432
    "}
  );
}

#[test]
fn test_insert_key_after() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document
    .insert_into("database.ssl", "true", InsertPosition::After("host".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        ssl: true
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_insert_key_before() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document
    .insert_into("database.ssl", "true", InsertPosition::Before("port".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        ssl: true
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_insert_key_preserves_comments() {
  let mut document = parse(indoc! {"
    # Config
    database:
      host: localhost
      port: 5432
    # End
  "});

  document
    .insert_into("database.ssl", "true", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Config
      database:
        host: localhost
        port: 5432
        ssl: true
      # End
    "}
  );
}

#[test]
fn test_insert_duplicate_key_errors() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document
    .insert_into("database.host", "newvalue", InsertPosition::Last)
    .is_err());
}

#[test]
fn test_insert_root_level_key() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  document.insert_into("ssl", "true", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      port: 5432
      ssl: true
    "}
  );
}

#[test]
fn test_insert_key_after_nonexistent_errors() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
  "});

  assert!(document
    .insert_into("database.ssl", "true", InsertPosition::After("missing".to_string()),)
    .is_err());
}

#[test]
fn test_insert_from_sort_order_middle() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      name: myapp
  "});

  let order = vec!["host".to_string(), "port".to_string(), "name".to_string()];

  document
    .insert_into("database.port", "5432", InsertPosition::FromSortOrder(order))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_insert_from_sort_order_first() {
  let mut document = parse(indoc! {"
    database:
      port: 5432
      name: myapp
  "});

  let order = vec!["host".to_string(), "port".to_string(), "name".to_string()];

  document
    .insert_into("database.host", "localhost", InsertPosition::FromSortOrder(order))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_insert_from_sort_order_key_not_in_order() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  let order = vec!["host".to_string(), "port".to_string()];

  document
    .insert_into("database.ssl", "true", InsertPosition::FromSortOrder(order))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        ssl: true
    "}
  );
}

#[test]
fn test_insert_into_nested_map() {
  let mut document = parse(indoc! {"
    settings:
      debug: true
  "});

  document
    .insert_into("settings.db_host", "localhost", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      settings:
        debug: true
        db_host: localhost
    "}
  );
}

#[test]
fn test_insert_into_deeply_nested_map() {
  let mut document = parse(indoc! {"
    a:
      b:
        c: 1
  "});

  document.insert_into("a.b.d", "2", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      a:
        b:
          c: 1
          d: 2
    "}
  );
}

#[test]
fn test_insert_into_sequence_at_end() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.insert_into("tags", "yaml", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - rust
        - yaml
    "}
  );
}

#[test]
fn test_insert_into_sequence_at_index() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.insert_into("tags", "yaml", InsertPosition::At(0)).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - yaml
        - ruby
        - rust
    "}
  );
}

#[test]
fn test_insert_into_sequence_before() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document
    .insert_into("tags", "yaml", InsertPosition::Before("rust".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - yaml
        - rust
    "}
  );
}

#[test]
fn test_insert_into_sequence_after() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document
    .insert_into("tags", "yaml", InsertPosition::After("ruby".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - yaml
        - rust
    "}
  );
}

#[test]
fn test_insert_with_bracket_index_path() {
  let mut document = parse(indoc! {"
    - id: first
      speakers:
        - Alice
    - id: second
      speakers:
        - Bob
  "});

  document
    .insert_into("[1].speakers", "Charlie", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        speakers:
          - Alice
      - id: second
        speakers:
          - Bob
          - Charlie
    "}
  );
}

#[test]
fn test_insert_with_bracket_index_after() {
  let mut document = parse(indoc! {"
    - id: first
      speakers:
        - Alice
        - Charlie
  "});

  document
    .insert_into("[0].speakers", "Bob", InsertPosition::After("Alice".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        speakers:
          - Alice
          - Bob
          - Charlie
    "}
  );
}

#[test]
fn test_insert_with_bracket_index_before() {
  let mut document = parse(indoc! {"
    - id: first
      speakers:
        - Alice
        - Charlie
  "});

  document
    .insert_into("[0].speakers", "Bob", InsertPosition::Before("Charlie".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        speakers:
          - Alice
          - Bob
          - Charlie
    "}
  );
}
