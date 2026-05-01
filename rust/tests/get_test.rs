use yerba::Document;

#[test]
fn test_get_plain_scalar() {
  let document = Document::parse("host: localhost").unwrap();

  assert_eq!(document.get("host"), Some("localhost".to_string()));
}

#[test]
fn test_get_double_quoted_scalar() {
  let document = Document::parse("name: \"myapp\"").unwrap();

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_single_quoted_scalar() {
  let document = Document::parse("name: 'myapp'").unwrap();

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_nested_path() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get("database.host"), Some("localhost".to_string()));
  assert_eq!(document.get("database.port"), Some("5432".to_string()));
}

#[test]
fn test_get_deeply_nested() {
  let yaml = "a:\n  b:\n    c: deep\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get("a.b.c"), Some("deep".to_string()));
}

#[test]
fn test_get_nonexistent_path() {
  let document = Document::parse("host: localhost").unwrap();

  assert_eq!(document.get("missing"), None);
}

#[test]
fn test_get_with_brackets_returns_first() {
  let yaml = "- id: a\n- id: b\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get("[].id"), Some("a".to_string()));
}

#[test]
fn test_get_all_with_bracket_path() {
  let yaml = "- id: a\n  name: first\n- id: b\n  name: second\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get_all("[].name"), vec!["first", "second"]);
  assert_eq!(document.get_all("[].id"), vec!["a", "b"]);
}

#[test]
fn test_get_all_nested_brackets() {
  let yaml = "- speakers:\n    - name: Alice\n    - name: Bob\n- speakers:\n    - name: Charlie\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get_all("[].speakers[].name"), vec!["Alice", "Bob", "Charlie"]);
}

#[test]
fn test_get_all_with_key_prefix() {
  let yaml = "data:\n  items:\n    - name: first\n    - name: second\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get_all("data.items.[].name"), vec!["first", "second"]);
}

#[test]
fn test_get_sequence_values() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get_sequence_values("tags"), vec!["ruby", "rust", "yaml"]);
}

#[test]
fn test_get_sequence_values_empty() {
  let yaml = "host: localhost\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.get_sequence_values("host").is_empty());
  assert!(document.get_sequence_values("missing").is_empty());
}

#[test]
fn test_exists_returns_true_for_existing_path() {
  let document = Document::parse("host: localhost\n").unwrap();

  assert!(document.exists("host"));
}

#[test]
fn test_exists_returns_false_for_missing_path() {
  let document = Document::parse("host: localhost\n").unwrap();

  assert!(!document.exists("missing"));
}

#[test]
fn test_exists_nested_path() {
  let document = Document::parse("database:\n  host: localhost\n").unwrap();

  assert!(document.exists("database.host"));
  assert!(!document.exists("database.missing"));
}

#[test]
fn test_exists_with_brackets() {
  let yaml = "- id: a\n  name: first\n- id: b\n  name: second\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.exists("[].name"));
  assert!(!document.exists("[].missing"));
}

#[test]
fn test_roundtrip_no_changes() {
  let yaml = "# comment\nkey: value\nnested:\n  a: 1\n  b: 'two'\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.to_string(), yaml);
}
