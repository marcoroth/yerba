use yerba::Document;

#[test]
fn test_append_to_sequence() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document.append("tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - ruby\n  - rust\n  - yaml\n"
  );
}

#[test]
fn test_append_to_nested_sequence() {
  let yaml = "app:\n  tags:\n    - ruby\n    - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document.append("app.tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    "app:\n  tags:\n    - ruby\n    - rust\n    - yaml\n"
  );
}

#[test]
fn test_append_preserves_comments() {
  let yaml = "# Tags\ntags:\n  - ruby\n  - rust\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.append("tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    "# Tags\ntags:\n  - ruby\n  - rust\n  - yaml\n# End\n"
  );
}

#[test]
fn test_remove_from_sequence() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.remove("tags", "rust").unwrap();

  assert_eq!(document.to_string(), "tags:\n  - ruby\n  - yaml\n");
}

#[test]
fn test_remove_from_nested_sequence() {
  let yaml = "app:\n  tags:\n    - ruby\n    - rust\n    - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.remove("app.tags", "rust").unwrap();

  assert_eq!(
    document.to_string(),
    "app:\n  tags:\n    - ruby\n    - yaml\n"
  );
}

#[test]
fn test_remove_nonexistent_item() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document.remove("tags", "missing").is_err());
}

#[test]
fn test_remove_from_non_sequence() {
  let mut document = Document::parse("host: localhost\n").unwrap();

  assert!(document.remove("host", "value").is_err());
}
