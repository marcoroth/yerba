mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_delete_key() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      name: myapp
    "}
  );
}

#[test]
fn test_delete_nested_key() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document.delete("database.port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        name: myapp
    "}
  );
}

#[test]
fn test_delete_preserves_comments() {
  let mut document = parse(indoc! {"
    # Config
    host: localhost
    port: 5432
    # End
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Config
      host: localhost
      # End
    "}
  );
}

#[test]
fn test_delete_nonexistent_key() {
  let mut document = parse("host: localhost\n");

  assert!(document.delete("missing").is_err());
}

#[test]
fn test_delete_orphans_trailing_comment() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    # port docs
    name: myapp
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      # port docs
      name: myapp
    "}
  );
}

#[test]
fn test_delete_preserves_inline_comment_on_own_line() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432 # important
    name: myapp
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      # important
      name: myapp
    "}
  );
}

#[test]
fn test_delete_with_bracket_index_path() {
  let mut document = parse(indoc! {"
    - id: first
      title: A
      description: remove me
    - id: second
      title: B
  "});

  document.delete("[0].description").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        title: A
      - id: second
        title: B
    "}
  );
}

#[test]
fn test_delete_root_sequence_item_by_index() {
  let mut document = parse(indoc! {"
    - name: Entry 1
    - name: Entry 2
      description: remove me too
  "});

  document.delete("[1]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Entry 1
    "}
  );
}

#[test]
fn test_delete_nested_sequence_item_by_index() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - go
  "});

  document.delete("tags[1]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - go
    "}
  );
}
