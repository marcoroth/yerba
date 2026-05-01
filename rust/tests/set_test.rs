use yerba::Document;

#[test]
fn test_set_plain_scalar() {
  let mut document = Document::parse("host: localhost").unwrap();

  document.set("host", "0.0.0.0").unwrap();

  assert_eq!(document.to_string(), "host: 0.0.0.0");
}

#[test]
fn test_set_preserves_double_quotes() {
  let mut document = Document::parse("name: \"myapp\"").unwrap();

  document.set("name", "newapp").unwrap();

  assert_eq!(document.to_string(), "name: \"newapp\"");
}

#[test]
fn test_set_preserves_single_quotes() {
  let mut document = Document::parse("name: 'myapp'").unwrap();

  document.set("name", "newapp").unwrap();

  assert_eq!(document.to_string(), "name: 'newapp'");
}

#[test]
fn test_set_nested_path() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("database.host", "0.0.0.0").unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: 0.0.0.0\n  port: 5432\n"
  );
}

#[test]
fn test_set_preserves_comments() {
  let yaml = "# Database config\nhost: localhost\n# Port\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("host", "0.0.0.0").unwrap();

  assert_eq!(
    document.to_string(),
    "# Database config\nhost: 0.0.0.0\n# Port\nport: 5432\n"
  );
}

#[test]
fn test_set_escapes_double_quotes_in_double_quoted_field() {
  let mut document = Document::parse("title: \"old title\"\n").unwrap();

  document.set("title", "something \"quoted\" here").unwrap();

  assert_eq!(
    document.to_string(),
    "title: \"something \\\"quoted\\\" here\"\n"
  );
}

#[test]
fn test_set_escapes_single_quotes_in_single_quoted_field() {
  let mut document = Document::parse("title: 'old title'\n").unwrap();

  document.set("title", "it's a test").unwrap();

  assert_eq!(document.to_string(), "title: 'it''s a test'\n");
}

#[test]
fn test_set_plain_field_with_value_containing_quotes() {
  let mut document = Document::parse("title: old\n").unwrap();

  document.set("title", "something \"quoted\"").unwrap();

  assert_eq!(document.to_string(), "title: something \"quoted\"\n");
}
