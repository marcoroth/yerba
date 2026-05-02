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

  assert_eq!(document.to_string(), "database:\n  host: 0.0.0.0\n  port: 5432\n");
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

  assert_eq!(document.to_string(), "title: \"something \\\"quoted\\\" here\"\n");
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

#[test]
fn test_set_with_bracket_index_path() {
  let yaml = "- id: first\n  title: A\n- id: second\n  title: B\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("[1].title", "Updated").unwrap();

  assert_eq!(
    document.to_string(),
    "- id: first\n  title: A\n- id: second\n  title: Updated\n"
  );
}

#[test]
fn test_set_with_bracket_index_first_item() {
  let yaml = "- id: first\n  title: A\n- id: second\n  title: B\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("[0].title", "Updated").unwrap();

  assert_eq!(
    document.to_string(),
    "- id: first\n  title: Updated\n- id: second\n  title: B\n"
  );
}

#[test]
fn test_set_block_scalar_to_empty() {
  let yaml = "- id: talk-1\n  description: |-\n    Some long description\n    across multiple lines\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("[0].description", "").unwrap();

  assert_eq!(document.to_string(), "- id: talk-1\n  description: \"\"\n");
}

#[test]
fn test_set_block_scalar_to_new_value() {
  let yaml = "description: |-\n  Old description\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("description", "New value").unwrap();

  assert_eq!(document.to_string(), "description: \"New value\"\n");
}

#[test]
fn test_set_bracket_index_out_of_bounds() {
  let yaml = "- id: first\n";
  let mut document = Document::parse(yaml).unwrap();

  let result = document.set("[5].id", "test");
  assert!(result.is_err());
}

#[test]
fn test_set_bracket_index_multiple_matches_error() {
  let yaml = "- id: first\n  title: A\n- id: second\n  title: B\n";
  let mut document = Document::parse(yaml).unwrap();

  let result = document.set("[].title", "test");
  assert!(
    result.is_err(),
    "setting on [] (all items) should error — use a specific index"
  );
}
