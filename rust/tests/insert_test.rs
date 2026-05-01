use yerba::{Document, InsertPosition};

#[test]
fn test_insert_key_at_end() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("database.ssl", "true", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  port: 5432\n  ssl: true\n"
  );
}

#[test]
fn test_insert_key_at_index() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("database.ssl", "true", InsertPosition::At(0))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  ssl: true\n  host: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_insert_key_after() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into(
      "database.ssl",
      "true",
      InsertPosition::After("host".to_string()),
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  ssl: true\n  port: 5432\n  name: myapp\n"
  );
}

#[test]
fn test_insert_key_before() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into(
      "database.ssl",
      "true",
      InsertPosition::Before("port".to_string()),
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  ssl: true\n  port: 5432\n  name: myapp\n"
  );
}

#[test]
fn test_insert_key_preserves_comments() {
  let yaml = "# Config\ndatabase:\n  host: localhost\n  port: 5432\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("database.ssl", "true", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "# Config\ndatabase:\n  host: localhost\n  port: 5432\n  ssl: true\n# End\n"
  );
}

#[test]
fn test_insert_duplicate_key_errors() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document
    .insert_into("database.host", "newvalue", InsertPosition::Last)
    .is_err());
}

#[test]
fn test_insert_root_level_key() {
  let yaml = "host: localhost\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("ssl", "true", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "host: localhost\nport: 5432\nssl: true\n"
  );
}

#[test]
fn test_insert_key_after_nonexistent_errors() {
  let yaml = "database:\n  host: localhost\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document
    .insert_into(
      "database.ssl",
      "true",
      InsertPosition::After("missing".to_string()),
    )
    .is_err());
}

#[test]
fn test_insert_from_sort_order_middle() {
  let yaml = "database:\n  host: localhost\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  let order = vec!["host".to_string(), "port".to_string(), "name".to_string()];

  document
    .insert_into(
      "database.port",
      "5432",
      InsertPosition::FromSortOrder(order),
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  port: 5432\n  name: myapp\n"
  );
}

#[test]
fn test_insert_from_sort_order_first() {
  let yaml = "database:\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  let order = vec!["host".to_string(), "port".to_string(), "name".to_string()];

  document
    .insert_into(
      "database.host",
      "localhost",
      InsertPosition::FromSortOrder(order),
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  port: 5432\n  name: myapp\n"
  );
}

#[test]
fn test_insert_from_sort_order_key_not_in_order() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  let order = vec!["host".to_string(), "port".to_string()];

  document
    .insert_into("database.ssl", "true", InsertPosition::FromSortOrder(order))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  port: 5432\n  ssl: true\n"
  );
}

#[test]
fn test_insert_into_nested_map() {
  let yaml = "settings:\n  debug: true\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("settings.db_host", "localhost", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "settings:\n  debug: true\n  db_host: localhost\n"
  );
}

#[test]
fn test_insert_into_deeply_nested_map() {
  let yaml = "a:\n  b:\n    c: 1\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("a.b.d", "2", InsertPosition::Last)
    .unwrap();

  assert_eq!(document.to_string(), "a:\n  b:\n    c: 1\n    d: 2\n");
}

#[test]
fn test_insert_into_sequence_at_end() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("tags", "yaml", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - ruby\n  - rust\n  - yaml\n"
  );
}

#[test]
fn test_insert_into_sequence_at_index() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("tags", "yaml", InsertPosition::At(0))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - yaml\n  - ruby\n  - rust\n"
  );
}

#[test]
fn test_insert_into_sequence_before() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("tags", "yaml", InsertPosition::Before("rust".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - ruby\n  - yaml\n  - rust\n"
  );
}

#[test]
fn test_insert_into_sequence_after() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .insert_into("tags", "yaml", InsertPosition::After("ruby".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - ruby\n  - yaml\n  - rust\n"
  );
}
