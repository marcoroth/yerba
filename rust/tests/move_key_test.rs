use yerba::Document;

#[test]
fn test_move_key_by_name() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("database", 2, 0).unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  name: myapp\n  host: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_move_key_forward() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("database", 0, 2).unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  port: 5432\n  name: myapp\n  host: localhost\n"
  );
}

#[test]
fn test_move_key_preserves_comments() {
  let yaml = "# Config\ndatabase:\n  host: localhost\n  port: 5432\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("database", 1, 0).unwrap();

  assert_eq!(
    document.to_string(),
    "# Config\ndatabase:\n  port: 5432\n  host: localhost\n# End\n"
  );
}

#[test]
fn test_move_key_out_of_bounds() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document.move_key("database", 5, 0).is_err());
}

#[test]
fn test_move_key_same_index_noop() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("database", 0, 0).unwrap();

  assert_eq!(document.to_string(), "database:\n  host: localhost\n  port: 5432\n");
}

#[test]
fn test_move_key_root_level() {
  let yaml = "host: localhost\nport: 5432\nname: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("", 2, 0).unwrap();

  assert_eq!(document.to_string(), "name: myapp\nhost: localhost\nport: 5432\n");
}

#[test]
fn test_resolve_key_index_by_name() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_key_index("database", "host").unwrap(), 0);
  assert_eq!(document.resolve_key_index("database", "port").unwrap(), 1);
  assert_eq!(document.resolve_key_index("database", "name").unwrap(), 2);
}

#[test]
fn test_resolve_key_index_by_number() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_key_index("database", "0").unwrap(), 0);
  assert_eq!(document.resolve_key_index("database", "1").unwrap(), 1);
}

#[test]
fn test_move_key_by_resolve_index() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let document = Document::parse(yaml).unwrap();

  let from = document.resolve_key_index("database", "name").unwrap();
  let to = document.resolve_key_index("database", "host").unwrap();

  assert_eq!(from, 2);
  assert_eq!(to, 0);
}

#[test]
fn test_resolve_key_index_nonexistent() {
  let yaml = "database:\n  host: localhost\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.resolve_key_index("database", "missing").is_err());
}

#[test]
fn test_resolve_key_index_out_of_bounds() {
  let yaml = "database:\n  host: localhost\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.resolve_key_index("database", "99").is_err());
}
