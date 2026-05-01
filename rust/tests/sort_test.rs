use yerba::{Document, SortField};

#[test]
fn test_sort_simple_array() {
  let yaml = "tags:\n  - rust\n  - yaml\n  - ruby\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("tags", &[], true).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - ruby\n  - rust\n  - yaml\n");
}

#[test]
fn test_sort_already_sorted_noop() {
  let yaml = "tags:\n  - a\n  - b\n  - c\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("tags", &[], true).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - a\n  - b\n  - c\n");
}

#[test]
fn test_sort_by_field() {
  let yaml = "- name: Charlie\n- name: Alice\n- name: Bob\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("", &[SortField::asc("name")], true).unwrap();

  assert_eq!(document.to_string(), "- name: Alice\n- name: Bob\n- name: Charlie\n");
}

#[test]
fn test_sort_by_field_descending() {
  let yaml = "- name: Alice\n- name: Bob\n- name: Charlie\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("", &[SortField::desc("name")], true).unwrap();

  assert_eq!(document.to_string(), "- name: Charlie\n- name: Bob\n- name: Alice\n");
}

#[test]
fn test_sort_by_multiple_fields() {
  let yaml = "- kind: talk\n  title: Zebra\n- kind: keynote\n  title: Alpha\n- kind: talk\n  title: Alpha\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .sort_items("", &[SortField::asc("kind"), SortField::asc("title")], true)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "- kind: keynote\n  title: Alpha\n- kind: talk\n  title: Alpha\n- kind: talk\n  title: Zebra\n"
  );
}

#[test]
fn test_sort_by_mixed_directions() {
  let yaml = "- kind: talk\n  title: B\n- kind: keynote\n  title: A\n- kind: talk\n  title: A\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .sort_items("", &[SortField::asc("kind"), SortField::desc("title")], true)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "- kind: keynote\n  title: A\n- kind: talk\n  title: B\n- kind: talk\n  title: A\n"
  );
}

#[test]
fn test_sort_nested_sequence() {
  let yaml = "data:\n  items:\n    - c\n    - a\n    - b\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("data.items", &[], true).unwrap();

  assert_eq!(document.to_string(), "data:\n  items:\n    - a\n    - b\n    - c\n");
}

#[test]
fn test_sort_each_nested_sequence() {
  let yaml = "- tags:\n    - rust\n    - ruby\n- tags:\n    - yaml\n    - go\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("[].tags", &[], true).unwrap();

  assert_eq!(
    document.to_string(),
    "- tags:\n    - ruby\n    - rust\n- tags:\n    - go\n    - yaml\n"
  );
}

#[test]
fn test_sort_single_item_noop() {
  let yaml = "tags:\n  - only\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("tags", &[], true).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - only\n");
}

#[test]
fn test_sort_field_parse() {
  let field = SortField::parse("title");
  assert_eq!(field.path, "title");
  assert!(field.ascending);

  let field = SortField::parse("date:desc");
  assert_eq!(field.path, "date");
  assert!(!field.ascending);
}

#[test]
fn test_sort_field_parse_list() {
  let fields = SortField::parse_list("kind,date:desc,title");

  assert_eq!(fields.len(), 3);
  assert_eq!(fields[0].path, "kind");
  assert!(fields[0].ascending);
  assert_eq!(fields[1].path, "date");
  assert!(!fields[1].ascending);
  assert_eq!(fields[2].path, "title");
  assert!(fields[2].ascending);
}

#[test]
fn test_sort_preserves_comments() {
  let yaml = "# tags\ntags:\n  - rust\n  - ruby\n# end\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("tags", &[], true).unwrap();

  assert_eq!(document.to_string(), "# tags\ntags:\n  - ruby\n  - rust\n# end\n");
}

#[test]
fn test_sort_case_insensitive() {
  let yaml = "- name: charlie\n- name: Alice\n- name: Bob\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("", &[SortField::asc("name")], false).unwrap();

  assert_eq!(document.to_string(), "- name: Alice\n- name: Bob\n- name: charlie\n");
}

#[test]
fn test_sort_case_sensitive() {
  let yaml = "- name: charlie\n- name: Alice\n- name: Bob\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("", &[SortField::asc("name")], true).unwrap();

  assert_eq!(document.to_string(), "- name: Alice\n- name: Bob\n- name: charlie\n");
}

#[test]
fn test_sort_case_insensitive_simple_array() {
  let yaml = "tags:\n  - Rust\n  - apple\n  - Banana\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_items("tags", &[], false).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - apple\n  - Banana\n  - Rust\n");
}
