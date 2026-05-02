mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_find_items_by_equality() {
  let document = parse(indoc! {"
    - kind: keynote
      title: Opening
    - kind: talk
      title: Testing
    - kind: keynote
      title: Closing
  "});

  let results = document.find_items("[]", ".kind == keynote");

  assert_eq!(results.len(), 2);
  assert!(results[0].text.contains("Opening"));
  assert!(results[1].text.contains("Closing"));
}

#[test]
fn test_find_items_by_contains() {
  let document = parse(indoc! {"
    - title: Talk A
      speakers:
        - Alice
        - Bob
    - title: Talk B
      speakers:
        - Charlie
  "});

  let results = document.find_items("[]", ".speakers contains \"Alice\"");

  assert_eq!(results.len(), 1);
  assert!(results[0].text.contains("Talk A"));
}

#[test]
fn test_find_items_no_matches() {
  let document = parse(indoc! {"
    - kind: talk
      title: A
    - kind: talk
      title: B
  "});

  let results = document.find_items("[]", ".kind == keynote");

  assert!(results.is_empty());
}

#[test]
fn test_find_items_nested_path() {
  let document = parse(indoc! {"
    conferences:
      - name: RailsConf
        year: 2024
      - name: RubyKaigi
        year: 2025
  "});

  let results = document.find_items("conferences.[]", ".year == 2024");

  assert_eq!(results.len(), 1);
  assert!(results[0].text.contains("RailsConf"));
}

#[test]
fn test_find_items_by_not_equals() {
  let document = parse(indoc! {"
    - kind: keynote
      title: A
    - kind: talk
      title: B
    - kind: talk
      title: C
  "});

  let results = document.find_items("[]", ".kind != keynote");

  assert_eq!(results.len(), 2);
  assert!(results[0].text.contains("title: B"));
  assert!(results[1].text.contains("title: C"));
}

#[test]
fn test_find_items_returns_line_numbers() {
  let document = parse(indoc! {"
    - id: first
      title: A
    - id: second
      title: B
    - id: third
      title: C
  "});

  let results = document.find_items("[]", ".id == second");

  assert_eq!(results.len(), 1);
  assert_eq!(results[0].line, 3);
}

#[test]
fn test_find_all_returns_line_numbers() {
  let document = parse(indoc! {"
    - id: first
    - id: second
    - id: third
  "});

  let results = document.find_all("[]");

  assert_eq!(results.len(), 3);
  assert_eq!(results[0].line, 1);
  assert_eq!(results[1].line, 2);
  assert_eq!(results[2].line, 3);
}
