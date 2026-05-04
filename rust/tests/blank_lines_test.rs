mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_add_blank_lines_between_entries() {
  let mut document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        title: First

      - id: b
        title: Second
    "}
  );
}

#[test]
fn test_remove_blank_lines_between_entries() {
  let mut document = parse(indoc! {"
    - id: a
      title: First

    - id: b
      title: Second
  "});

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        title: First
      - id: b
        title: Second
    "}
  );
}

#[test]
fn test_blank_lines_noop_when_already_correct() {
  let mut document = parse(indoc! {"
    - id: a

    - id: b
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a

      - id: b
    "}
  );
}

#[test]
fn test_blank_lines_nested_sequence() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  document.enforce_blank_lines("tags", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby

        - rust

        - yaml
    "}
  );
}

#[test]
fn test_blank_lines_single_entry_noop() {
  let mut document = parse(indoc! {"
    - id: only
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(document.to_string(), "- id: only\n");
}

#[test]
fn test_blank_lines_multiple_blank_lines() {
  let mut document = parse(indoc! {"
    - id: a
    - id: b
    - id: c
  "});

  document.enforce_blank_lines("", 2).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a


      - id: b


      - id: c
    "}
  );
}

#[test]
fn test_blank_lines_preserves_comments() {
  let mut document = parse(indoc! {"
    # comment
    - id: a
    - id: b
    # end
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # comment
      - id: a

      - id: b
      # end
    "}
  );
}

#[test]
fn test_blank_lines_with_multi_line_entries() {
  let mut document = parse(indoc! {"
    - id: a
      title: First
      speakers:
        - Alice
    - id: b
      title: Second
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        title: First
        speakers:
          - Alice

      - id: b
        title: Second
    "}
  );
}

#[test]
fn test_blank_lines_normalizes_extra_blanks() {
  let mut document = parse(indoc! {"
    - id: a



    - id: b
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a

      - id: b
    "}
  );
}

#[test]
fn test_blank_lines_top_level_only() {
  let mut document = parse(indoc! {"
    - id: a
      speakers:
        - Alice
        - Bob
    - id: b
      speakers:
        - Charlie
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        speakers:
          - Alice
          - Bob

      - id: b
        speakers:
          - Charlie
    "}
  );
}

#[test]
fn test_blank_lines_nested_only() {
  let mut document = parse(indoc! {"
    - id: a
      speakers:
        - Alice
        - Bob
    - id: b
      speakers:
        - Charlie
        - Diana
  "});

  document.enforce_blank_lines("[].speakers", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        speakers:
          - Alice

          - Bob
      - id: b
        speakers:
          - Charlie

          - Diana
    "}
  );
}

#[test]
fn test_blank_lines_remove_from_map() {
  let mut document = parse(indoc! {"
    host: localhost

    port: 5432

    name: myapp
  "});

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      port: 5432
      name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_add_to_map() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost

      port: 5432

      name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_map_noop_when_correct() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  let original = document.to_string();

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_blank_lines_nested_map() {
  let mut document = parse(indoc! {"
    database:
      host: localhost

      port: 5432

      name: myapp
  "});

  document.enforce_blank_lines("database", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_map_with_comments() {
  let mut document = parse(indoc! {"
    # Database config
    host: localhost

    port: 5432

    # App name
    name: myapp
  "});

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Database config
      host: localhost
      port: 5432

      # App name
      name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_map_normalizes_multiple() {
  let mut document = parse(indoc! {"
    host: localhost



    port: 5432
  "});

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      port: 5432
    "}
  );
}
