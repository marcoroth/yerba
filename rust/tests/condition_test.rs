use yerba::Document;

#[test]
fn test_evaluate_condition_equal() {
  let document = Document::parse("database:\n  host: localhost\n  port: 5432\n").unwrap();

  assert!(document.evaluate_condition("database", ".host == \"localhost\""));
  assert!(!document.evaluate_condition("database", ".host == \"other\""));
}

#[test]
fn test_evaluate_condition_not_equal() {
  let document = Document::parse("database:\n  host: localhost\n  port: 5432\n").unwrap();

  assert!(document.evaluate_condition("database", ".host != \"other\""));
  assert!(!document.evaluate_condition("database", ".host != \"localhost\""));
}

#[test]
fn test_evaluate_condition_with_number() {
  let document = Document::parse("database:\n  host: localhost\n  port: 5432\n").unwrap();

  assert!(document.evaluate_condition("database", ".port == \"5432\""));
  assert!(document.evaluate_condition("database", ".port == 5432"));
}

#[test]
fn test_evaluate_condition_single_quoted_value() {
  let document = Document::parse("database:\n  host: localhost\n  port: 5432\n").unwrap();

  assert!(document.evaluate_condition("database", ".host == 'localhost'"));
}

#[test]
fn test_evaluate_condition_unquoted_value() {
  let document = Document::parse("database:\n  host: localhost\n  port: 5432\n").unwrap();

  assert!(document.evaluate_condition("database", ".host == localhost"));
}

#[test]
fn test_evaluate_condition_root_level() {
  let document = Document::parse("host: localhost\nport: 5432\n").unwrap();

  assert!(document.evaluate_condition("", ".host == \"localhost\""));
  assert!(document.evaluate_condition("", ".port == 5432"));
}

#[test]
fn test_evaluate_condition_missing_key_returns_false() {
  let document = Document::parse("host: localhost\n").unwrap();

  assert!(!document.evaluate_condition("", ".missing == \"value\""));
}

#[test]
fn test_evaluate_condition_boolean_value() {
  let document = Document::parse("app:\n  debug: true\n  verbose: false\n").unwrap();

  assert!(document.evaluate_condition("app", ".debug == true"));
  assert!(document.evaluate_condition("app", ".verbose == false"));
  assert!(!document.evaluate_condition("app", ".debug == false"));
}

#[test]
fn test_evaluate_condition_quoted_yaml_value() {
  let document = Document::parse("name: \"myapp\"\n").unwrap();

  assert!(document.evaluate_condition("", ".name == \"myapp\""));
  assert!(document.evaluate_condition("", ".name == myapp"));
}

#[test]
fn test_evaluate_condition_invalid_condition() {
  let document = Document::parse("host: localhost\n").unwrap();

  assert!(!document.evaluate_condition("", "not a condition"));
  assert!(!document.evaluate_condition("", ""));
}

#[test]
fn test_evaluate_condition_contains() {
  let yaml = "app:\n  tags:\n    - ruby\n    - rust\n    - yaml\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.evaluate_condition("app", ".tags contains \"ruby\""));
  assert!(document.evaluate_condition("app", ".tags contains rust"));
  assert!(!document.evaluate_condition("app", ".tags contains python"));
}

#[test]
fn test_evaluate_condition_not_contains() {
  let yaml = "app:\n  tags:\n    - ruby\n    - rust\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.evaluate_condition("app", ".tags not_contains python"));
  assert!(!document.evaluate_condition("app", ".tags not_contains ruby"));
}

#[test]
fn test_evaluate_condition_contains_root_level() {
  let yaml = "speakers:\n  - Marco Roth\n  - Nadia Odunayo\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.evaluate_condition("", ".speakers contains \"Marco Roth\""));
  assert!(!document.evaluate_condition("", ".speakers contains \"Unknown\""));
}

#[test]
fn test_evaluate_condition_contains_empty_sequence() {
  let yaml = "tags:\n  - ruby\n";
  let document = Document::parse(yaml).unwrap();

  assert!(!document.evaluate_condition("", ".missing contains ruby"));
}

#[test]
fn test_evaluate_condition_nested_bracket_contains() {
  let yaml = "talks:\n  - speakers:\n      - name: Marco Roth\n      - name: Nadia\n  - speakers:\n      - name: Alice\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.evaluate_condition("", ".talks[].speakers[].name contains \"Marco Roth\""));
  assert!(!document.evaluate_condition("", ".talks[].speakers[].name contains \"Unknown\""));
}

#[test]
fn test_evaluate_condition_bracket_equals() {
  let yaml = "- kind: keynote\n  title: A\n- kind: talk\n  title: B\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.evaluate_condition("", ".[].kind == keynote"));
  assert!(document.evaluate_condition("", ".[].kind == talk"));
  assert!(!document.evaluate_condition("", ".[].kind == workshop"));
}

#[test]
fn test_evaluate_condition_bracket_not_equals() {
  let yaml = "- kind: keynote\n- kind: keynote\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.evaluate_condition("", ".[].kind != talk"));
  assert!(!document.evaluate_condition("", ".[].kind != keynote"));
}

#[test]
fn test_evaluate_condition_flat_speakers_contains() {
  let yaml = "speakers:\n  - Marco Roth\n  - Nadia Odunayo\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.evaluate_condition("", ".speakers contains \"Marco Roth\""));
  assert!(document.evaluate_condition("", ".speakers[] contains \"Marco Roth\""));
}
