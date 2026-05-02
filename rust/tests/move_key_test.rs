mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_move_key_by_name() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document.move_key("database", 2, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        name: myapp
        host: localhost
        port: 5432
    "}
  );
}

#[test]
fn test_move_key_forward() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document.move_key("database", 0, 2).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        port: 5432
        name: myapp
        host: localhost
    "}
  );
}

#[test]
fn test_move_key_preserves_comments() {
  let mut document = parse(indoc! {"
    # Config
    database:
      host: localhost
      port: 5432
    # End
  "});

  document.move_key("database", 1, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Config
      database:
        port: 5432
        host: localhost
      # End
    "}
  );
}

#[test]
fn test_move_key_out_of_bounds() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.move_key("database", 5, 0).is_err());
}

#[test]
fn test_move_key_same_index_noop() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document.move_key("database", 0, 0).unwrap();

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
fn test_move_key_root_level() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  document.move_key("", 2, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: myapp
      host: localhost
      port: 5432
    "}
  );
}

#[test]
fn test_resolve_key_index_by_name() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  assert_eq!(document.resolve_key_index("database", "host").unwrap(), 0);
  assert_eq!(document.resolve_key_index("database", "port").unwrap(), 1);
  assert_eq!(document.resolve_key_index("database", "name").unwrap(), 2);
}

#[test]
fn test_resolve_key_index_by_number() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert_eq!(document.resolve_key_index("database", "0").unwrap(), 0);
  assert_eq!(document.resolve_key_index("database", "1").unwrap(), 1);
}

#[test]
fn test_move_key_by_resolve_index() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  let from = document.resolve_key_index("database", "name").unwrap();
  let to = document.resolve_key_index("database", "host").unwrap();

  assert_eq!(from, 2);
  assert_eq!(to, 0);
}

#[test]
fn test_resolve_key_index_nonexistent() {
  let document = parse(indoc! {"
    database:
      host: localhost
  "});

  assert!(document.resolve_key_index("database", "missing").is_err());
}

#[test]
fn test_resolve_key_index_out_of_bounds() {
  let document = parse(indoc! {"
    database:
      host: localhost
  "});

  assert!(document.resolve_key_index("database", "99").is_err());
}

#[test]
fn test_move_key_preserves_trailing_comment() {
  let mut document = parse(indoc! {"
    id: test
    slides_url: https://example.com
    # source link
    title: Test
  "});

  document.move_key("", 1, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      slides_url: https://example.com
      # source link
      id: test
      title: Test
    "}
  );
}

#[test]
fn test_move_key_preserves_inline_comment() {
  let mut document = parse(indoc! {r#"
    id: test
    title: "Lightning Talks" # TODO: use cues
    description: ""
  "#});

  document.move_key("", 1, 2).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      id: test
      description: ""
      title: "Lightning Talks" # TODO: use cues
    "#}
  );
}
