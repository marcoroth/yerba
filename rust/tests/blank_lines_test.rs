use yerba::Document;

#[test]
fn test_add_blank_lines_between_entries() {
  let yaml = "- id: a\n  title: First\n- id: b\n  title: Second\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    "- id: a\n  title: First\n\n- id: b\n  title: Second\n"
  );
}

#[test]
fn test_remove_blank_lines_between_entries() {
  let yaml = "- id: a\n  title: First\n\n- id: b\n  title: Second\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    "- id: a\n  title: First\n- id: b\n  title: Second\n"
  );
}

#[test]
fn test_blank_lines_noop_when_already_correct() {
  let yaml = "- id: a\n\n- id: b\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(document.to_string(), "- id: a\n\n- id: b\n");
}

#[test]
fn test_blank_lines_nested_sequence() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("tags", 1).unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - ruby\n\n  - rust\n\n  - yaml\n"
  );
}

#[test]
fn test_blank_lines_single_entry_noop() {
  let yaml = "- id: only\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(document.to_string(), "- id: only\n");
}

#[test]
fn test_blank_lines_multiple_blank_lines() {
  let yaml = "- id: a\n- id: b\n- id: c\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 2).unwrap();

  assert_eq!(document.to_string(), "- id: a\n\n\n- id: b\n\n\n- id: c\n");
}

#[test]
fn test_blank_lines_preserves_comments() {
  let yaml = "# comment\n- id: a\n- id: b\n# end\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    "# comment\n- id: a\n\n- id: b\n# end\n"
  );
}

#[test]
fn test_blank_lines_with_multi_line_entries() {
  let yaml = "- id: a\n  title: First\n  speakers:\n    - Alice\n- id: b\n  title: Second\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    "- id: a\n  title: First\n  speakers:\n    - Alice\n\n- id: b\n  title: Second\n"
  );
}

#[test]
fn test_blank_lines_normalizes_extra_blanks() {
  let yaml = "- id: a\n\n\n\n- id: b\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(document.to_string(), "- id: a\n\n- id: b\n");
}
