mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_sort_keys() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
      pool: 10
  "});

  document
    .sort_keys("database", &["name", "host", "port", "pool"])
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        name: myapp
        host: localhost
        port: 5432
        pool: 10
    "}
  );
}

#[test]
fn test_sort_keys_partial_order() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
      pool: 10
  "});

  document.sort_keys("database", &["name", "pool"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        name: myapp
        pool: 10
        host: localhost
        port: 5432
    "}
  );
}

#[test]
fn test_sort_keys_with_bracket_path() {
  let mut document = parse(indoc! {"
    - name: first
      id: 1
    - name: second
      id: 2
  "});

  document.sort_keys("[]", &["id", "name"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: 1
        name: first
      - id: 2
        name: second
    "}
  );
}

#[test]
fn test_validate_sort_keys_with_bracket_path() {
  let mut document = parse(indoc! {"
    - name: first
      id: 1
  "});

  assert!(document.validate_sort_keys("[]", &["id", "name"]).is_ok());
  assert!(document.validate_sort_keys("[]", &["id"]).is_err());
}

#[test]
fn test_sort_keys_preserves_trailing_comments() {
  let mut document = parse(indoc! {"
    id: test
    # comment after id
    title: Test
    description: foo
  "});

  document.sort_keys("", &["title", "id", "description"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      title: Test
      id: test
      # comment after id
      description: foo
    "}
  );
}

#[test]
fn test_sort_keys_preserves_multiple_trailing_comments() {
  let mut document = parse(indoc! {"
    id: test
    # first comment
    # second comment
    title: Test
  "});

  document.sort_keys("", &["title", "id"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      title: Test
      id: test
      # first comment
      # second comment
    "}
  );
}

#[test]
fn test_sort_keys_preserves_inline_comments() {
  let mut document = parse(indoc! {r#"
    - title: "Lightning Talks" # TODO: use cues
      id: lt-1
      speakers:
        - Alice
  "#});

  document.sort_keys("[]", &["id", "title", "speakers"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: lt-1
        title: "Lightning Talks" # TODO: use cues
        speakers:
          - Alice
    "#}
  );
}

#[test]
fn test_sort_keys_preserves_trailing_comments_in_sequence() {
  let mut document = parse(indoc! {"
    - id: talk-1
      slides_url: https://example.com
      # https://x.com/status/123
      title: Test
  "});

  document.sort_keys("[]", &["id", "title", "slides_url"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        title: Test
        slides_url: https://example.com
        # https://x.com/status/123
    "}
  );
}

#[test]
fn test_sort_keys_preserves_comments_between_sequence_entries() {
  let mut document = parse(indoc! {r#"
    - id: "first"
      title: "Hello"
      published_at: "2024-01-01"

    # Section Comment

    - id: "second"
      title: "World"
      published_at: "2024-01-02"
  "#});

  document.sort_keys("[]", &["id", "title", "published_at"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: "first"
        title: "Hello"
        published_at: "2024-01-01"

      # Section Comment

      - id: "second"
        title: "World"
        published_at: "2024-01-02"
    "#}
  );
}
