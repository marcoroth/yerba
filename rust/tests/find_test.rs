use yerba::Document;

#[test]
fn test_find_items_by_equality() {
  let yaml = "- kind: keynote\n  title: Opening\n- kind: talk\n  title: Testing\n- kind: keynote\n  title: Closing\n";
  let document = Document::parse(yaml).unwrap();

  let results = document.find_items("[]", ".kind == keynote");

  assert_eq!(results.len(), 2);
  assert!(results[0].text.contains("Opening"));
  assert!(results[1].text.contains("Closing"));
}

#[test]
fn test_find_items_by_contains() {
  let yaml = "- title: Talk A\n  speakers:\n    - Alice\n    - Bob\n- title: Talk B\n  speakers:\n    - Charlie\n";
  let document = Document::parse(yaml).unwrap();

  let results = document.find_items("[]", ".speakers contains \"Alice\"");

  assert_eq!(results.len(), 1);
  assert!(results[0].text.contains("Talk A"));
}

#[test]
fn test_find_items_no_matches() {
  let yaml = "- kind: talk\n  title: A\n- kind: talk\n  title: B\n";
  let document = Document::parse(yaml).unwrap();

  let results = document.find_items("[]", ".kind == keynote");

  assert!(results.is_empty());
}

#[test]
fn test_find_items_nested_path() {
  let yaml = "conferences:\n  - name: RailsConf\n    year: 2024\n  - name: RubyKaigi\n    year: 2025\n";
  let document = Document::parse(yaml).unwrap();

  let results = document.find_items("conferences.[]", ".year == 2024");

  assert_eq!(results.len(), 1);
  assert!(results[0].text.contains("RailsConf"));
}

#[test]
fn test_find_items_by_not_equals() {
  let yaml = "- kind: keynote\n  title: A\n- kind: talk\n  title: B\n- kind: talk\n  title: C\n";
  let document = Document::parse(yaml).unwrap();

  let results = document.find_items("[]", ".kind != keynote");

  assert_eq!(results.len(), 2);
  assert!(results[0].text.contains("title: B"));
  assert!(results[1].text.contains("title: C"));
}

#[test]
fn test_find_items_returns_line_numbers() {
  let yaml = "- id: first\n  title: A\n- id: second\n  title: B\n- id: third\n  title: C\n";
  let document = Document::parse(yaml).unwrap();

  let results = document.find_items("[]", ".id == second");

  assert_eq!(results.len(), 1);
  assert_eq!(results[0].line, 3);
}

#[test]
fn test_find_all_returns_line_numbers() {
  let yaml = "- id: first\n- id: second\n- id: third\n";
  let document = Document::parse(yaml).unwrap();

  let results = document.find_all("[]");

  assert_eq!(results.len(), 3);
  assert_eq!(results[0].line, 1);
  assert_eq!(results[1].line, 2);
  assert_eq!(results[2].line, 3);
}
