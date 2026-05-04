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

  document.sort_keys("database", &["name", "host", "port", "pool"]).unwrap();

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
  let document = parse(indoc! {"
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

#[test]
fn test_sort_keys_does_not_add_blank_lines_between_map_entries() {
  let mut document = parse(indoc! {r#"
    - id: "first"
      speakers:
        - Alice
      description: |-
        Some long text
        across multiple lines
      video_provider: "youtube"
      published_at: "2024-01-01"

    - id: "second"
      speakers:
        - Bob
      description: ""
      video_provider: "youtube"
      published_at: "2024-01-02"
  "#});

  document
    .sort_keys("[]", &["id", "published_at", "video_provider", "description", "speakers"])
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: "first"
        published_at: "2024-01-01"
        video_provider: "youtube"
        description: |-
          Some long text
          across multiple lines
        speakers:
          - Alice

      - id: "second"
        published_at: "2024-01-02"
        video_provider: "youtube"
        description: ""
        speakers:
          - Bob
    "#}
  );
}

#[test]
fn test_sort_keys_with_nested_sequence_path() {
  let mut document = parse(indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
    - title: "Panel"
      talks:
        - speakers:
            - Bob
          title: "Sub Talk"
        - speakers:
            - Charlie
          title: "Another"
  "#});

  document.sort_keys("[].talks[]", &["title", "speakers"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
    - title: "Panel"
      talks:
        - title: "Sub Talk"
          speakers:
            - Bob
        - title: "Another"
          speakers:
            - Charlie
  "#}
  );
}

#[test]
fn test_sort_keys_with_nested_sequence_skips_missing() {
  let mut document = parse(indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
    - title: "Panel"
      talks:
        - speakers:
            - Bob
          title: "Sub Talk"
  "#});

  document.sort_keys("[].talks[]", &["title", "speakers"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
    - title: "Panel"
      talks:
        - title: "Sub Talk"
          speakers:
            - Bob
  "#}
  );
}

#[test]
fn test_sort_keys_with_deeply_nested_sequence() {
  let mut document = parse(indoc! {r#"
    - title: "Panel"
      talks:
        - title: "Sub Talk"
          alternative_recordings:
            - speakers:
                - Bob
              title: "Alt Recording"
  "#});

  document.sort_keys("[].talks[].alternative_recordings[]", &["title", "speakers"]).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - title: "Panel"
      talks:
        - title: "Sub Talk"
          alternative_recordings:
            - title: "Alt Recording"
              speakers:
                - Bob
  "#}
  );
}

#[test]
fn test_sort_keys_on_file_without_selector() {
  let mut document = parse(indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
  "#});

  let result = document.sort_keys("[].talks[]", &["title", "speakers"]);

  assert!(result.is_ok());
  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
  "#}
  );
}

#[test]
fn test_validate_sort_keys_on_file_without_selector() {
  let document = parse(indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
  "#});

  let result = document.validate_sort_keys("[].talks[]", &["title", "speakers"]);

  assert!(result.is_ok());
}

#[test]
fn test_validate_sort_keys_with_nested_sequence_skips_missing() {
  let document = parse(indoc! {r#"
    - title: "Keynote"
      speakers:
        - Alice
    - title: "Panel"
      talks:
        - title: "Sub Talk"
          speakers:
            - Bob
  "#});

  let result = document.validate_sort_keys("[].talks[]", &["title", "speakers"]);

  assert!(result.is_ok());
}
