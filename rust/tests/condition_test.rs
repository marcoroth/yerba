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

  assert!(document.evaluate_condition("database", ".host == \"localhost\""));
  assert!(!document.evaluate_condition("database", ".host == \"other\""));
}

#[test]
fn test_evaluate_condition_not_equal() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host != \"other\""));
  assert!(!document.evaluate_condition("database", ".host != \"localhost\""));
}

#[test]
fn test_evaluate_condition_with_number() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".port == \"5432\""));
  assert!(document.evaluate_condition("database", ".port == 5432"));
}

#[test]
fn test_evaluate_condition_single_quoted_value() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host == 'localhost'"));
}

#[test]
fn test_evaluate_condition_unquoted_value() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host == localhost"));
}

#[test]
fn test_evaluate_condition_root_level() {
  let document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  assert!(document.evaluate_condition("", ".host == \"localhost\""));
  assert!(document.evaluate_condition("", ".port == 5432"));
}

#[test]
fn test_evaluate_condition_missing_key_returns_false() {
  let document = parse("host: localhost\n");

  assert!(!document.evaluate_condition("", ".missing == \"value\""));
}

#[test]
fn test_evaluate_condition_boolean_value() {
  let document = parse(indoc! {"
    app:
      debug: true
      verbose: false
  "});

  assert!(document.evaluate_condition("app", ".debug == true"));
  assert!(document.evaluate_condition("app", ".verbose == false"));
  assert!(!document.evaluate_condition("app", ".debug == false"));
}

#[test]
fn test_evaluate_condition_quoted_yaml_value() {
  let document = parse(indoc! {r#"
    name: "myapp"
  "#});

  assert!(document.evaluate_condition("", ".name == \"myapp\""));
  assert!(document.evaluate_condition("", ".name == myapp"));
}

#[test]
fn test_evaluate_condition_invalid_condition() {
  let document = parse("host: localhost\n");

  assert!(!document.evaluate_condition("", "not a condition"));
  assert!(!document.evaluate_condition("", ""));
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

  assert!(document.evaluate_condition("app", ".tags contains \"ruby\""));
  assert!(document.evaluate_condition("app", ".tags contains rust"));
  assert!(!document.evaluate_condition("app", ".tags contains python"));
}

#[test]
fn test_evaluate_condition_not_contains() {
  let document = parse(indoc! {"
    app:
      tags:
        - ruby
        - rust
  "});

  assert!(document.evaluate_condition("app", ".tags not_contains python"));
  assert!(!document.evaluate_condition("app", ".tags not_contains ruby"));
}

#[test]
fn test_evaluate_condition_contains_root_level() {
  let document = parse(indoc! {"
    speakers:
      - Marco Roth
      - Nadia Odunayo
  "});

  assert!(document.evaluate_condition("", ".speakers contains \"Marco Roth\""));
  assert!(!document.evaluate_condition("", ".speakers contains \"Unknown\""));
}

#[test]
fn test_evaluate_condition_contains_empty_sequence() {
  let document = parse(indoc! {"
    tags:
      - ruby
  "});

  assert!(!document.evaluate_condition("", ".missing contains ruby"));
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

  assert!(document.evaluate_condition("", ".talks[].speakers[].name contains \"Marco Roth\""));
  assert!(!document.evaluate_condition("", ".talks[].speakers[].name contains \"Unknown\""));
}

#[test]
fn test_evaluate_condition_bracket_equals() {
  let document = parse(indoc! {"
    - kind: keynote
      title: A
    - kind: talk
      title: B
  "});

  assert!(document.evaluate_condition("", ".[].kind == keynote"));
  assert!(document.evaluate_condition("", ".[].kind == talk"));
  assert!(!document.evaluate_condition("", ".[].kind == workshop"));
}

#[test]
fn test_evaluate_condition_bracket_not_equals() {
  let document = parse(indoc! {"
    - kind: keynote
    - kind: keynote
  "});

  assert!(document.evaluate_condition("", ".[].kind != talk"));
  assert!(!document.evaluate_condition("", ".[].kind != keynote"));
}

#[test]
fn test_evaluate_condition_flat_speakers_contains() {
  let document = parse(indoc! {"
    speakers:
      - Marco Roth
      - Nadia Odunayo
  "});

  assert!(document.evaluate_condition("", ".speakers contains \"Marco Roth\""));
  assert!(document.evaluate_condition("", ".speakers[] contains \"Marco Roth\""));
}

#[test]
fn test_evaluate_condition_contains_substring() {
  let document = parse("title: Ruby on Rails\n");

  assert!(document.evaluate_condition("", ".title contains Ruby"));
  assert!(document.evaluate_condition("", ".title contains Rails"));
  assert!(document.evaluate_condition("", ".title contains \"on\""));
  assert!(!document.evaluate_condition("", ".title contains Python"));
}

#[test]
fn test_evaluate_condition_not_contains_substring() {
  let document = parse("title: Ruby on Rails\n");

  assert!(document.evaluate_condition("", ".title not_contains Python"));
  assert!(!document.evaluate_condition("", ".title not_contains Ruby"));
}

#[test]
fn test_evaluate_condition_contains_still_works_for_arrays() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  assert!(document.evaluate_condition("", ".tags contains ruby"));
  assert!(!document.evaluate_condition("", ".tags contains rub"));
}

#[test]
fn test_evaluate_condition_contains_substring_nested() {
  let document = parse(indoc! {"
    database:
      host: localhost
      name: myapp_development
  "});

  assert!(document.evaluate_condition("database", ".name contains development"));
  assert!(document.evaluate_condition("database", ".name contains myapp"));
  assert!(!document.evaluate_condition("database", ".name contains production"));
}

#[test]
fn test_find_items_requires_dot_prefix() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: keynote
    - id: talk-2
      kind: talk
  "});

  let matches = document.find_items("[]", ".kind == keynote");
  assert_eq!(matches.len(), 1);

  let matches = document.find_items("[]", "kind == keynote");
  assert_eq!(matches.len(), 0);
}

#[test]
fn test_filter_values_returns_structured_values() {
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

  let values = document.filter_values("[]", ".kind == keynote");
  assert_eq!(values.len(), 2);

  if let serde_yaml::Value::Mapping(map) = &values[0] {
    assert_eq!(
      map.get(&serde_yaml::Value::String("title".to_string())),
      Some(&serde_yaml::Value::String("Opening".to_string()))
    );
  } else {
    panic!("Expected Mapping, got: {:?}", values[0]);
  }
}

#[test]
fn test_filter_values_empty_on_no_match() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: talk
  "});

  let values = document.filter_values("[]", ".kind == keynote");
  assert!(values.is_empty());
}

#[test]
fn test_evaluate_condition_relative_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("database", ".host == localhost"));
  assert!(!document.evaluate_condition("database", ".host == remotehost"));
}

#[test]
fn test_evaluate_condition_absolute_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.evaluate_condition("", "database.host == localhost"));
}
