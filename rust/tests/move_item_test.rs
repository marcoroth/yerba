use yerba::Document;

#[test]
fn test_move_item_by_index() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 2, 0).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - yaml\n  - ruby\n  - rust\n");
}

#[test]
fn test_move_item_forward() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 0, 2).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - rust\n  - yaml\n  - ruby\n");
}

#[test]
fn test_move_item_same_index() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 0, 0).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - ruby\n  - rust\n");
}

#[test]
fn test_move_item_out_of_bounds() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document.move_item("tags", 5, 0).is_err());
}

#[test]
fn test_move_item_preserves_comments() {
  let yaml = "# Tags\ntags:\n  - ruby\n  - rust\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 1, 0).unwrap();

  assert_eq!(document.to_string(), "# Tags\ntags:\n  - rust\n  - ruby\n# End\n");
}

#[test]
fn test_resolve_sequence_index_by_name() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_sequence_index("tags", "ruby").unwrap(), 0);
  assert_eq!(document.resolve_sequence_index("tags", "rust").unwrap(), 1);
  assert_eq!(document.resolve_sequence_index("tags", "yaml").unwrap(), 2);
}

#[test]
fn test_resolve_sequence_index_by_number() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_sequence_index("tags", "0").unwrap(), 0);
  assert_eq!(document.resolve_sequence_index("tags", "1").unwrap(), 1);
}
