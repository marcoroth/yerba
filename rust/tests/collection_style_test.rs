mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_get_collection_style_block_sequence() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rails
  "});

  assert_eq!(document.get_collection_style("tags"), Some("block"));
}

#[test]
fn test_get_collection_style_flow_sequence() {
  let document = parse(indoc! {"
    tags: [ruby, rails]
  "});

  assert_eq!(document.get_collection_style("tags"), Some("flow"));
}

#[test]
fn test_get_collection_style_block_map() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert_eq!(document.get_collection_style("database"), Some("block"));
}

#[test]
fn test_get_collection_style_flow_map() {
  let document = parse(indoc! {"
    database: {host: localhost, port: 5432}
  "});

  assert_eq!(document.get_collection_style("database"), Some("flow"));
}

#[test]
fn test_get_collection_style_scalar_returns_none() {
  let document = parse(indoc! {"
    name: Alice
  "});

  assert_eq!(document.get_collection_style("name"), None);
}

#[test]
fn test_set_collection_style_flow_to_block_sequence() {
  let mut document = parse(indoc! {"
    tags: [ruby, rails]
  "});

  document.set_collection_style("tags", "block").unwrap();

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
fn test_set_collection_style_block_to_flow_sequence() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rails
  "});

  document.set_collection_style("tags", "flow").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags: [ruby, rails]
    "}
  );
}

#[test]
fn test_set_collection_style_flow_to_block_map() {
  let mut document = parse(indoc! {"
    database: {host: localhost, port: 5432}
  "});

  document.set_collection_style("database", "block").unwrap();

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
fn test_set_collection_style_block_to_flow_map() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document.set_collection_style("database", "flow").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database: {host: localhost, port: 5432}
    "}
  );
}

#[test]
fn test_set_collection_style_same_style_is_noop() {
  let mut document = parse(indoc! {"
    tags: [ruby, rails]
  "});

  document.set_collection_style("tags", "flow").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags: [ruby, rails]
    "}
  );
}

#[test]
fn test_set_collection_style_preserves_surrounding_content() {
  let mut document = parse(indoc! {"
    name: Alice
    tags: [ruby, rails]
    age: 30
  "});

  document.set_collection_style("tags", "block").unwrap();

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
fn test_insert_map_key_with_multiline_value() {
  let mut document = parse(indoc! {"
    name: Alice
  "});

  document.insert_into("tags", "- ruby\n- rails", yerba::InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: Alice
      tags:
        - ruby
        - rails
    "}
  );
}
