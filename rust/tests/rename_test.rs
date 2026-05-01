use yerba::Document;

#[test]
fn test_rename_key_same_level() {
  let yaml = "host: localhost\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("host", "hostname").unwrap();

  assert_eq!(document.to_string(), "hostname: localhost\nport: 5432\n");
}

#[test]
fn test_rename_nested_key_same_level() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("database.host", "database.hostname").unwrap();

  assert_eq!(document.to_string(), "database:\n  hostname: localhost\n  port: 5432\n");
}

#[test]
fn test_rename_preserves_comments() {
  let yaml = "# Config\nhost: localhost\n# Port\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("host", "hostname").unwrap();

  assert_eq!(
    document.to_string(),
    "# Config\nhostname: localhost\n# Port\nport: 5432\n"
  );
}

#[test]
fn test_rename_preserves_quoted_key() {
  let yaml = "'host': localhost\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("host", "hostname").unwrap();

  assert_eq!(document.to_string(), "'hostname': localhost\n");
}

#[test]
fn test_rename_nonexistent_key() {
  let mut document = Document::parse("host: localhost\n").unwrap();

  assert!(document.rename("missing", "new_name").is_err());
}

#[test]
fn test_rename_relocate_to_root() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("database.host", "hostname").unwrap();

  assert_eq!(document.to_string(), "database:\n  port: 5432\nhostname: localhost\n");
}

#[test]
fn test_rename_relocate_to_other_map() {
  let yaml = "database:\n  host: localhost\n  port: 5432\nsettings:\n  debug: true\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("database.host", "settings.db_host").unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  port: 5432\nsettings:\n  debug: true\n  db_host: localhost\n"
  );
}

#[test]
fn test_rename_relocate_from_root_to_nested() {
  let yaml = "hostname: localhost\ndatabase:\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("hostname", "database.host").unwrap();

  assert_eq!(document.to_string(), "database:\n  port: 5432\n  host: localhost\n");
}
