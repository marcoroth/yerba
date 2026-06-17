mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_append_to_sequence() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.append("tags", "yaml").unwrap();

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
fn test_append_to_empty_sequence() {
  let mut document = parse(indoc! {"
    tags: []
  "});

  document.append("tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - yaml
    "}
  );
}

#[test]
fn test_append_to_empty_sequence_preserves_inline_comment() {
  let mut document = parse(indoc! {"
    tags: [] # Keep this comment
  "});

  document.append("tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags: # Keep this comment
        - yaml
    "}
  );
}

#[test]
fn test_append_to_nested_sequence() {
  let mut document = parse(indoc! {"
    app:
      tags:
        - ruby
        - rust
  "});

  document.append("app.tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        tags:
          - ruby
          - rust
          - yaml
    "}
  );
}

#[test]
fn test_append_to_nested_empty_sequence() {
  let mut document = parse(indoc! {"
    app:
      tags: []
  "});

  document.append("app.tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        tags:
          - yaml
    "}
  );
}

#[test]
fn test_append_preserves_comments() {
  let mut document = parse(indoc! {"
    # Tags
    tags:
      - ruby
      - rust
    # End
  "});

  document.append("tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Tags
      tags:
        - ruby
        - rust
        - yaml
      # End
    "}
  );
}

#[test]
fn test_remove_from_sequence() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  document.remove("tags", "rust").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - yaml
    "}
  );
}

#[test]
fn test_remove_from_nested_sequence() {
  let mut document = parse(indoc! {"
    app:
      tags:
        - ruby
        - rust
        - yaml
  "});

  document.remove("app.tags", "rust").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        tags:
          - ruby
          - yaml
    "}
  );
}

#[test]
fn test_remove_nonexistent_item() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  assert!(document.remove("tags", "missing").is_err());
}

#[test]
fn test_remove_from_non_sequence() {
  let mut document = parse("host: localhost\n");

  assert!(document.remove("host", "value").is_err());
}

// Inline comments on a removed sequence item are preserved on their own line.
#[test]
fn test_remove_preserves_inline_comment_on_own_line() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust # important
      - yaml
  "});

  document.remove("tags", "rust").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        # important
        - yaml
    "}
  );
}
