mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_get_sequence_indent_indented() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rails
  "});

  assert_eq!(document.get_sequence_indent("tags"), Some("indented"));
}

#[test]
fn test_get_sequence_indent_compact() {
  let document = parse(indoc! {"
    tags:
    - ruby
    - rails
  "});

  assert_eq!(document.get_sequence_indent("tags"), Some("compact"));
}

#[test]
fn test_get_sequence_indent_scalar_returns_none() {
  let document = parse(indoc! {"
    name: Alice
  "});

  assert_eq!(document.get_sequence_indent("name"), None);
}

#[test]
fn test_get_sequence_indent_flow_returns_none() {
  let document = parse(indoc! {"
    tags: [ruby, rails]
  "});

  assert_eq!(document.get_sequence_indent("tags"), None);
}

#[test]
fn test_set_sequence_indent_indented_to_compact() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rails
  "});

  document.set_sequence_indent("tags", "compact").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
      - ruby
      - rails
    "}
  );
}

#[test]
fn test_set_sequence_indent_compact_to_indented() {
  let mut document = parse(indoc! {"
    tags:
    - ruby
    - rails
  "});

  document.set_sequence_indent("tags", "indented").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - rails
    "}
  );
}

#[test]
fn test_set_sequence_indent_same_style_is_noop() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rails
  "});

  document.set_sequence_indent("tags", "indented").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - rails
    "}
  );
}

#[test]
fn test_set_sequence_indent_preserves_surrounding_content() {
  let mut document = parse(indoc! {"
    name: Alice
    tags:
      - ruby
      - rails
    age: 30
  "});

  document.set_sequence_indent("tags", "compact").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: Alice
      tags:
      - ruby
      - rails
      age: 30
    "}
  );
}

#[test]
fn test_set_sequence_indent_nested_indented_to_compact() {
  let mut document = parse(indoc! {"
    app:
      tags:
        - ruby
        - rails
  "});

  document.set_sequence_indent("app.tags", "compact").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        tags:
        - ruby
        - rails
    "}
  );
}

#[test]
fn test_set_sequence_indent_nested_compact_to_indented() {
  let mut document = parse(indoc! {"
    app:
      tags:
      - ruby
      - rails
  "});

  document.set_sequence_indent("app.tags", "indented").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        tags:
          - ruby
          - rails
    "}
  );
}

#[test]
fn test_set_sequence_indent_with_nested_map_entries() {
  let mut document = parse(indoc! {"
    talks:
      - title: Talk 1
        speakers:
          - Alice
      - title: Talk 2
        speakers:
          - Bob
  "});

  document.set_sequence_indent("talks", "compact").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      talks:
      - title: Talk 1
        speakers:
          - Alice
      - title: Talk 2
        speakers:
          - Bob
    "}
  );
}

#[test]
fn test_set_sequence_indent_invalid_style_returns_error() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
  "});

  let result = document.set_sequence_indent("tags", "wide");

  assert!(result.is_err());
}
