use yerba::Document;

#[test]
fn test_delete_key() {
  let yaml = "host: localhost\nport: 5432\nname: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.delete("port").unwrap();

  assert_eq!(document.to_string(), "host: localhost\nname: myapp\n");
}

#[test]
fn test_delete_nested_key() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.delete("database.port").unwrap();

  assert_eq!(document.to_string(), "database:\n  host: localhost\n  name: myapp\n");
}

#[test]
fn test_delete_preserves_comments() {
  let yaml = "# Config\nhost: localhost\nport: 5432\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.delete("port").unwrap();

  assert_eq!(document.to_string(), "# Config\nhost: localhost\n# End\n");
}

#[test]
fn test_delete_nonexistent_key() {
  let mut document = Document::parse("host: localhost\n").unwrap();

  assert!(document.delete("missing").is_err());
}
