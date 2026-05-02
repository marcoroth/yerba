mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_get_plain_scalar() {
  let document = parse("host: localhost");

  assert_eq!(document.get("host"), Some("localhost".to_string()));
}

#[test]
fn test_get_double_quoted_scalar() {
  let document = parse(r#"name: "myapp""#);

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_single_quoted_scalar() {
  let document = parse("name: 'myapp'");

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_nested_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert_eq!(document.get("database.host"), Some("localhost".to_string()));
  assert_eq!(document.get("database.port"), Some("5432".to_string()));
}

#[test]
fn test_get_deeply_nested() {
  let document = parse(indoc! {"
    a:
      b:
        c: deep
  "});

  assert_eq!(document.get("a.b.c"), Some("deep".to_string()));
}

#[test]
fn test_get_nonexistent_path() {
  let document = parse("host: localhost");

  assert_eq!(document.get("missing"), None);
}

#[test]
fn test_get_with_brackets_returns_first() {
  let document = parse(indoc! {"
    - id: a
    - id: b
  "});

  assert_eq!(document.get("[].id"), Some("a".to_string()));
}

#[test]
fn test_get_all_with_bracket_path() {
  let document = parse(indoc! {"
    - id: a
      name: first
    - id: b
      name: second
  "});

  assert_eq!(document.get_all("[].name"), vec!["first", "second"]);
  assert_eq!(document.get_all("[].id"), vec!["a", "b"]);
}

#[test]
fn test_get_all_nested_brackets() {
  let document = parse(indoc! {"
    - speakers:
        - name: Alice
        - name: Bob
    - speakers:
        - name: Charlie
  "});

  assert_eq!(document.get_all("[].speakers[].name"), vec!["Alice", "Bob", "Charlie"]);
}

#[test]
fn test_get_all_with_key_prefix() {
  let document = parse(indoc! {"
    data:
      items:
        - name: first
        - name: second
  "});

  assert_eq!(document.get_all("data.items.[].name"), vec!["first", "second"]);
}

#[test]
fn test_get_sequence_values() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  assert_eq!(document.get_sequence_values("tags"), vec!["ruby", "rust", "yaml"]);
}

#[test]
fn test_get_sequence_values_empty() {
  let document = parse("host: localhost\n");

  assert!(document.get_sequence_values("host").is_empty());
  assert!(document.get_sequence_values("missing").is_empty());
}

#[test]
fn test_exists_returns_true_for_existing_path() {
  let document = parse("host: localhost\n");

  assert!(document.exists("host"));
}

#[test]
fn test_exists_returns_false_for_missing_path() {
  let document = parse("host: localhost\n");

  assert!(!document.exists("missing"));
}

#[test]
fn test_exists_nested_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
  "});

  assert!(document.exists("database.host"));
  assert!(!document.exists("database.missing"));
}

#[test]
fn test_exists_with_brackets() {
  let document = parse(indoc! {"
    - id: a
      name: first
    - id: b
      name: second
  "});

  assert!(document.exists("[].name"));
  assert!(!document.exists("[].missing"));
}

#[test]
fn test_roundtrip_no_changes() {
  let yaml = indoc! {"
    # comment
    key: value
    nested:
      a: 1
      b: 'two'
  "};
  let document = parse(yaml);

  assert_eq!(document.to_string(), yaml);
}

#[test]
fn test_get_with_bracket_index() {
  let document = parse(indoc! {"
    - id: first
      title: A
    - id: second
      title: B
  "});

  assert_eq!(document.get("[0].title"), Some("A".to_string()));
  assert_eq!(document.get("[1].title"), Some("B".to_string()));
  assert_eq!(document.get("[2].title"), None);
}

#[test]
fn test_get_nested_bracket_path() {
  let document = parse(indoc! {"
    - id: talk-1
      speakers:
        - Alice
        - Bob
    - id: talk-2
      speakers:
        - Charlie
  "});

  assert_eq!(document.get_all("[].speakers[]"), vec!["Alice", "Bob", "Charlie"]);
  assert_eq!(document.get_all("[0].speakers[]"), vec!["Alice", "Bob"]);
  assert_eq!(document.get_all("[1].speakers[]"), vec!["Charlie"]);
}
