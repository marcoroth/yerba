use yerba::Document;

#[test]
fn test_rename_key() {
  let yaml = "host: localhost\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("host", "hostname").unwrap();

  assert_eq!(document.to_string(), "hostname: localhost\nport: 5432\n");
}

#[test]
fn test_rename_nested_key() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("database.host", "hostname").unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  hostname: localhost\n  port: 5432\n"
  );
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
