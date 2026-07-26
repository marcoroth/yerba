mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_evaluate_condition_equal() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host == \"localhost\"").unwrap());
  assert!(!document.evaluate_condition("database", ".host == \"other\"").unwrap());
}

#[test]
fn test_evaluate_condition_not_equal() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host != \"other\"").unwrap());
  assert!(!document.evaluate_condition("database", ".host != \"localhost\"").unwrap());
}

#[test]
fn test_evaluate_condition_with_number() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".port == \"5432\"").unwrap());
  assert!(document.evaluate_condition("database", ".port == 5432").unwrap());
}

#[test]
fn test_evaluate_condition_single_quoted_value() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host == 'localhost'").unwrap());
}

#[test]
fn test_evaluate_condition_unquoted_value() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host == localhost").unwrap());
}

#[test]
fn test_evaluate_condition_root_level() {
  let document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  assert!(document.evaluate_condition("", ".host == \"localhost\"").unwrap());
  assert!(document.evaluate_condition("", ".port == 5432").unwrap());
}

#[test]
fn test_evaluate_condition_missing_key_returns_false() {
  let document = parse("host: localhost\n");

  assert!(!document.evaluate_condition("", ".missing == \"value\"").unwrap());
}

#[test]
fn test_evaluate_condition_boolean_value() {
  let document = parse(indoc! {"
    app:
      debug: true
      verbose: false
  "});

  assert!(document.evaluate_condition("app", ".debug == true").unwrap());
  assert!(document.evaluate_condition("app", ".verbose == false").unwrap());
  assert!(!document.evaluate_condition("app", ".debug == false").unwrap());
}

#[test]
fn test_evaluate_condition_quoted_yaml_value() {
  let document = parse(indoc! {r#"
    name: "myapp"
  "#});

  assert!(document.evaluate_condition("", ".name == \"myapp\"").unwrap());
  assert!(document.evaluate_condition("", ".name == myapp").unwrap());
}

#[test]
fn test_evaluate_condition_invalid_condition_is_rejected() {
  let document = parse("host: localhost\n");

  assert!(document.evaluate_condition("", "not a condition").is_err());
  assert!(document.evaluate_condition("", "").is_err());
  assert!(document.evaluate_condition("", ".host = localhost").is_err());
}

#[test]
fn test_evaluate_condition_absolute_path_rejected_under_a_parent() {
  let document = parse(indoc! {"
    database:
      host: localhost
  "});

  let error = document.evaluate_condition("database", "host == localhost").unwrap_err();

  assert!(error.to_string().contains("must be relative"), "{}", error);
}

#[test]
fn test_evaluate_condition_contains() {
  let document = parse(indoc! {"
    app:
      tags:
        - ruby
        - rust
        - yaml
  "});

  assert!(document.evaluate_condition("app", ".tags contains \"ruby\"").unwrap());
  assert!(document.evaluate_condition("app", ".tags contains rust").unwrap());
  assert!(!document.evaluate_condition("app", ".tags contains python").unwrap());
}

#[test]
fn test_evaluate_condition_not_contains() {
  let document = parse(indoc! {"
    app:
      tags:
        - ruby
        - rust
  "});

  assert!(document.evaluate_condition("app", ".tags not_contains python").unwrap());
  assert!(!document.evaluate_condition("app", ".tags not_contains ruby").unwrap());
}

#[test]
fn test_evaluate_condition_contains_root_level() {
  let document = parse(indoc! {"
    speakers:
      - Marco Roth
      - Nadia Odunayo
  "});

  assert!(document.evaluate_condition("", ".speakers contains \"Marco Roth\"").unwrap());
  assert!(!document.evaluate_condition("", ".speakers contains \"Unknown\"").unwrap());
}

#[test]
fn test_evaluate_condition_contains_empty_sequence() {
  let document = parse(indoc! {"
    tags:
      - ruby
  "});

  assert!(!document.evaluate_condition("", ".missing contains ruby").unwrap());
}

#[test]
fn test_evaluate_condition_nested_bracket_contains() {
  let document = parse(indoc! {"
    talks:
      - speakers:
          - name: Marco Roth
          - name: Nadia
      - speakers:
          - name: Alice
  "});

  assert!(document.evaluate_condition("", ".talks[].speakers[].name contains \"Marco Roth\"").unwrap());
  assert!(!document.evaluate_condition("", ".talks[].speakers[].name contains \"Unknown\"").unwrap());
}

#[test]
fn test_evaluate_condition_bracket_equals() {
  let document = parse(indoc! {"
    - kind: keynote
      title: A
    - kind: talk
      title: B
  "});

  assert!(document.evaluate_condition("", ".[].kind == keynote").unwrap());
  assert!(document.evaluate_condition("", ".[].kind == talk").unwrap());
  assert!(!document.evaluate_condition("", ".[].kind == workshop").unwrap());
}

#[test]
fn test_evaluate_condition_bracket_not_equals() {
  let document = parse(indoc! {"
    - kind: keynote
    - kind: keynote
  "});

  assert!(document.evaluate_condition("", ".[].kind != talk").unwrap());
  assert!(!document.evaluate_condition("", ".[].kind != keynote").unwrap());
}

#[test]
fn test_evaluate_condition_flat_speakers_contains() {
  let document = parse(indoc! {"
    speakers:
      - Marco Roth
      - Nadia Odunayo
  "});

  assert!(document.evaluate_condition("", ".speakers contains \"Marco Roth\"").unwrap());
  assert!(document.evaluate_condition("", ".speakers[] contains \"Marco Roth\"").unwrap());
}

#[test]
fn test_evaluate_condition_contains_substring() {
  let document = parse("title: Ruby on Rails\n");

  assert!(document.evaluate_condition("", ".title contains Ruby").unwrap());
  assert!(document.evaluate_condition("", ".title contains Rails").unwrap());
  assert!(document.evaluate_condition("", ".title contains \"on\"").unwrap());
  assert!(!document.evaluate_condition("", ".title contains Python").unwrap());
}

#[test]
fn test_evaluate_condition_not_contains_substring() {
  let document = parse("title: Ruby on Rails\n");

  assert!(document.evaluate_condition("", ".title not_contains Python").unwrap());
  assert!(!document.evaluate_condition("", ".title not_contains Ruby").unwrap());
}

#[test]
fn test_evaluate_condition_contains_still_works_for_arrays() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  assert!(document.evaluate_condition("", ".tags contains ruby").unwrap());
  assert!(!document.evaluate_condition("", ".tags contains rub").unwrap());
}

#[test]
fn test_evaluate_condition_contains_substring_nested() {
  let document = parse(indoc! {"
    database:
      host: localhost
      name: myapp_development
  "});

  assert!(document.evaluate_condition("database", ".name contains development").unwrap());
  assert!(document.evaluate_condition("database", ".name contains myapp").unwrap());
  assert!(!document.evaluate_condition("database", ".name contains production").unwrap());
}

#[test]
fn test_find_items_requires_dot_prefix() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: keynote
    - id: talk-2
      kind: talk
  "});

  let matches = document.filter("[]", ".kind == keynote").unwrap();
  assert_eq!(matches.len(), 1);

  assert!(document.filter("[]", "kind == keynote").is_err());
}

#[test]
fn test_filter_returns_structured_values() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: keynote
      title: Opening
    - id: talk-2
      kind: talk
      title: Deep Dive
    - id: talk-3
      kind: keynote
      title: Closing
  "});

  let values = document.filter("[]", ".kind == keynote").unwrap();
  assert_eq!(values.len(), 2);

  if let yaml_serde::Value::Mapping(map) = &values[0] {
    assert_eq!(
      map.get(&yaml_serde::Value::String("title".to_string())),
      Some(&yaml_serde::Value::String("Opening".to_string()))
    );
  } else {
    panic!("Expected Mapping, got: {:?}", values[0]);
  }
}

#[test]
fn test_filter_empty_on_no_match() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: talk
  "});

  let values = document.filter("[]", ".kind == keynote").unwrap();
  assert!(values.is_empty());
}

#[test]
fn test_evaluate_condition_relative_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host == localhost").unwrap());
  assert!(!document.evaluate_condition("database", ".host == remotehost").unwrap());
}

#[test]
fn test_evaluate_condition_absolute_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("", "database.host == localhost").unwrap());
}
