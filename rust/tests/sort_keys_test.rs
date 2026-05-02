use yerba::Document;

#[test]
fn test_sort_keys() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n  pool: 10\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .sort_keys("database", &["name", "host", "port", "pool"])
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  name: myapp\n  host: localhost\n  port: 5432\n  pool: 10\n"
  );
}

#[test]
fn test_sort_keys_partial_order() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n  pool: 10\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_keys("database", &["name", "pool"]).unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  name: myapp\n  pool: 10\n  host: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_sort_keys_with_bracket_path() {
  let yaml = "- name: first\n  id: 1\n- name: second\n  id: 2\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_keys("[]", &["id", "name"]).unwrap();

  assert_eq!(
    document.to_string(),
    "- id: 1\n  name: first\n- id: 2\n  name: second\n"
  );
}

#[test]
fn test_validate_sort_keys_with_bracket_path() {
  let yaml = "- name: first\n  id: 1\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.validate_sort_keys("[]", &["id", "name"]).is_ok());
  assert!(document.validate_sort_keys("[]", &["id"]).is_err());
}

#[test]
fn test_sort_keys_preserves_trailing_comments() {
  let yaml = "id: test\n# comment after id\ntitle: Test\ndescription: foo\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_keys("", &["title", "id", "description"]).unwrap();

  let result = document.to_string();
  assert!(
    result.contains("# comment after id"),
    "trailing comment should be preserved"
  );
  assert!(result.contains("id: test\n# comment after id"));
}

#[test]
fn test_sort_keys_preserves_multiple_trailing_comments() {
  let yaml = "id: test\n# first comment\n# second comment\ntitle: Test\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_keys("", &["title", "id"]).unwrap();

  let result = document.to_string();
  assert!(
    result.contains("# first comment"),
    "first trailing comment should be preserved"
  );
  assert!(
    result.contains("# second comment"),
    "second trailing comment should be preserved"
  );
}

#[test]
fn test_sort_keys_preserves_inline_comments() {
  let yaml = "- title: \"Lightning Talks\" # TODO: use cues\n  id: lt-1\n  speakers:\n    - Alice\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_keys("[]", &["id", "title", "speakers"]).unwrap();

  let result = document.to_string();
  assert!(
    result.contains("# TODO: use cues"),
    "inline comment should be preserved, got: {}",
    result
  );
}

#[test]
fn test_sort_keys_preserves_trailing_comments_in_sequence() {
  let yaml = "- id: talk-1\n  slides_url: https://example.com\n  # https://x.com/status/123\n  title: Test\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_keys("[]", &["id", "title", "slides_url"]).unwrap();

  let result = document.to_string();
  assert!(
    result.contains("# https://x.com/status/123"),
    "trailing comment should be preserved, got: {}",
    result
  );
}
