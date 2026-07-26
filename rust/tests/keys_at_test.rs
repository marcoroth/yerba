mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_keys_at_root() {
  let document = parse(indoc! {"
    name: Conf
    year: 2024
    venue:
      city: Berlin
  "});

  assert_eq!(document.keys_at(""), vec!["name", "year", "venue"]);
}

#[test]
fn test_keys_at_nested_map() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert_eq!(document.keys_at("database"), vec!["host", "port"]);
}

#[test]
fn test_keys_at_does_not_descend_into_nested_sequence() {
  let document = parse(indoc! {"
    id: aloha
    name: Aloha
    channels:
      - id: UC123
        handle: '@confreaks'
  "});

  assert_eq!(document.keys_at(""), vec!["id", "name", "channels"]);
}

#[test]
fn test_keys_at_reports_key_without_a_value() {
  let document = parse(indoc! {"
    a: 1
    b:
    c: 3
  "});

  assert_eq!(document.keys_at(""), vec!["a", "b", "c"]);
}

#[test]
fn test_keys_at_reads_flow_map() {
  let document = parse("venue: { city: Berlin, country: DE }");

  assert_eq!(document.keys_at("venue"), vec!["city", "country"]);
}

#[test]
fn test_keys_at_map_inside_sequence() {
  let document = parse(indoc! {"
    - id: a
      title: First
    - id: b
  "});

  assert_eq!(document.keys_at("[0]"), vec!["id", "title"]);
}

#[test]
fn test_keys_at_returns_nothing_for_a_sequence() {
  let document = parse(indoc! {"
    tags:
      - ruby
  "});

  assert!(document.keys_at("tags").is_empty());
}

#[test]
fn test_keys_at_returns_nothing_for_a_sequence_root() {
  let document = parse(indoc! {"
    - id: a
    - id: b
  "});

  assert!(document.keys_at("").is_empty());
}

#[test]
fn test_keys_at_returns_nothing_for_a_scalar() {
  let document = parse("name: Conf");

  assert!(document.keys_at("name").is_empty());
}

#[test]
fn test_keys_at_returns_nothing_for_a_missing_selector() {
  let document = parse("name: Conf");

  assert!(document.keys_at("missing").is_empty());
}
