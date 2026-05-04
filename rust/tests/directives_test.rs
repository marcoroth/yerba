mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_ensure_directives_adds_marker() {
  let mut document = parse(indoc! {"
    name: Alice
  "});

  document.ensure_directives().unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      ---
      name: Alice
    "}
  );
}

#[test]
fn test_ensure_directives_noop_when_present() {
  let mut document = parse(indoc! {"
    ---
    name: Alice
  "});

  let original = document.to_string();

  document.ensure_directives().unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_remove_directives() {
  let mut document = parse(indoc! {"
    ---
    name: Alice
  "});

  document.remove_directives().unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: Alice
    "}
  );
}

#[test]
fn test_remove_directives_noop_when_absent() {
  let mut document = parse(indoc! {"
    name: Alice
  "});

  let original = document.to_string();

  document.remove_directives().unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_has_directives_marker_true() {
  let document = parse(indoc! {"
    ---
    name: Alice
  "});

  assert!(document.has_directives_marker());
}

#[test]
fn test_has_directives_marker_false() {
  let document = parse(indoc! {"
    name: Alice
  "});

  assert!(!document.has_directives_marker());
}

#[test]
fn test_ensure_directives_preserves_content() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
    tags:
      - ruby
      - rust
  "});

  document.ensure_directives().unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      ---
      database:
        host: localhost
        port: 5432
      tags:
        - ruby
        - rust
    "}
  );
}

#[test]
fn test_remove_directives_preserves_content() {
  let mut document = parse(indoc! {"
    ---
    database:
      host: localhost
      port: 5432
  "});

  document.remove_directives().unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
    "}
  );
}

#[test]
fn test_ensure_then_remove_roundtrip() {
  let mut document = parse(indoc! {"
    name: Alice
  "});

  let original = document.to_string();

  document.ensure_directives().unwrap();
  assert!(document.has_directives_marker());

  document.remove_directives().unwrap();
  assert!(!document.has_directives_marker());

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_ensure_directives_with_sequence() {
  let mut document = parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  document.ensure_directives().unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      ---
      - id: talk-1
        title: First
      - id: talk-2
        title: Second
    "}
  );
}

#[test]
fn test_remove_directives_from_sequence() {
  let mut document = parse(indoc! {"
    ---
    - id: talk-1
      title: First
  "});

  document.remove_directives().unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        title: First
    "}
  );
}

#[test]
fn test_ensure_directives_with_comments() {
  let mut document = parse(indoc! {"
    # Config file
    name: Alice
  "});

  document.ensure_directives().unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      ---
      # Config file
      name: Alice
    "}
  );
}
