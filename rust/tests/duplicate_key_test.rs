use indoc::indoc;

#[test]
fn test_duplicate_key_at_root() {
  let result = yerba::parse(indoc! {"
    host: localhost
    port: 5432
    host: 0.0.0.0
  "});

  let error = result.unwrap_err();
  let message = format!("{}", error);

  assert!(message.contains("duplicate key \"host\""));
  assert!(message.contains("on line 3"));
  assert!(message.contains("first defined on line 1"));
}

#[test]
fn test_duplicate_key_in_nested_map() {
  let result = yerba::parse(indoc! {"
    database:
      host: localhost
      port: 5432
      host: 0.0.0.0
  "});

  let error = result.unwrap_err();
  let message = format!("{}", error);

  assert!(message.contains("duplicate key \"host\""));
  assert!(message.contains("on line 4"));
  assert!(message.contains("first defined on line 2"));
}

#[test]
fn test_duplicate_key_in_sequence_item() {
  let result = yerba::parse(indoc! {"
    - id: talk-1
      title: First
      id: duplicate
  "});

  let error = result.unwrap_err();
  let message = format!("{}", error);

  assert!(message.contains("duplicate key \"id\""));
}

#[test]
fn test_no_duplicate_keys_across_sequence_items() {
  let result = yerba::parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  assert!(result.is_ok());
}

#[test]
fn test_no_duplicate_keys_across_nested_maps() {
  let result = yerba::parse(indoc! {"
    database:
      host: localhost
    cache:
      host: redis
  "});

  assert!(result.is_ok());
}

#[test]
fn test_no_error_on_valid_file() {
  let result = yerba::parse(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  assert!(result.is_ok());
}

#[test]
fn test_duplicate_key_shows_line_content() {
  let result = yerba::parse(indoc! {"
    host: localhost
    port: 5432
    host: 0.0.0.0
  "});

  let error = result.unwrap_err();
  let message = format!("{}", error);

  assert!(message.contains("host: 0.0.0.0"));
}
