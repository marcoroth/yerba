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
fn test_get_collection_style_nested_block_sequence() {
  let document = parse(indoc! {"
    app:
      tags:
        - ruby
        - rails
  "});

  assert_eq!(document.get_collection_style("app.tags"), Some("block"));
}

#[test]
fn test_get_collection_style_nested_flow_sequence() {
  let document = parse(indoc! {"
    app:
      tags: [ruby, rails]
  "});

  assert_eq!(document.get_collection_style("app.tags"), Some("flow"));
}

#[test]
fn test_set_collection_style_nested_flow_to_block() {
  let mut document = parse(indoc! {"
    app:
      name: MyApp
      tags: [ruby, rails]
      version: 1
  "});

  document.set_collection_style("app.tags", "block").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        name: MyApp
        tags:
          - ruby
          - rails
        version: 1
    "}
  );
}

#[test]
fn test_set_collection_style_nested_block_to_flow() {
  let mut document = parse(indoc! {"
    app:
      name: MyApp
      tags:
        - ruby
        - rails
      version: 1
  "});

  document.set_collection_style("app.tags", "flow").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        name: MyApp
        tags: [ruby, rails]
        version: 1
    "}
  );
}

#[test]
fn test_set_collection_style_flow_to_block_nested_map() {
  let mut document = parse(indoc! {"
    app:
      database: {host: localhost, port: 5432}
  "});

  document.set_collection_style("app.database", "block").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        database:
          host: localhost
          port: 5432
    "}
  );
}

#[test]
fn test_set_collection_style_invalid_style_returns_error() {
  let mut document = parse(indoc! {"
    tags: [ruby, rails]
  "});

  let result = document.set_collection_style("tags", "compact");

  assert!(result.is_err());
}

#[test]
fn test_set_collection_style_on_scalar_returns_error() {
  let mut document = parse(indoc! {"
    name: Alice
  "});

  let result = document.set_collection_style("name", "block");

  assert!(result.is_err());
}

#[test]
fn test_set_collection_style_skips_empty_flow_sequence() {
  let mut document = parse(indoc! {"
    talks: []
  "});

  document.set_collection_style("talks", "block").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      talks: []
    "}
  );
}

#[test]
fn test_set_collection_style_skips_empty_flow_map() {
  let mut document = parse(indoc! {"
    config: {}
  "});

  document.set_collection_style("config", "block").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      config: {}
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

#[test]
fn test_enforce_styles_combined_collection_and_quote() {
  use yerba::document::style::StyleEnforcement;

  let mut document = parse(indoc! {"
    tags: [ruby, rails]
    name: localhost
  "});

  document
    .enforce_styles(&StyleEnforcement {
      collection_style: Some("block".to_string()),
      sequence_indent: None,
      key_style: None,
      value_style: Some(yerba::QuoteStyle::Double),
    })
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      tags:
        - ruby
        - rails
      name: "localhost"
    "#}
  );
}

#[test]
fn test_enforce_styles_combined_all() {
  use yerba::document::style::StyleEnforcement;

  let mut document = parse(indoc! {"
    tags: [ruby, rails]
    name: localhost
  "});

  document
    .enforce_styles(&StyleEnforcement {
      collection_style: Some("block".to_string()),
      sequence_indent: Some("indented".to_string()),
      key_style: Some(yerba::KeyStyle::Plain),
      value_style: Some(yerba::QuoteStyle::Double),
    })
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      tags:
        - ruby
        - rails
      name: "localhost"
    "#}
  );
}

#[test]
fn test_enforce_styles_noop_when_all_match() {
  use yerba::document::style::StyleEnforcement;

  let mut document = parse(indoc! {r#"
    tags:
      - ruby
      - rails
    name: "localhost"
  "#});

  let original = document.to_string();

  document
    .enforce_styles(&StyleEnforcement {
      collection_style: Some("block".to_string()),
      sequence_indent: Some("indented".to_string()),
      key_style: Some(yerba::KeyStyle::Plain),
      value_style: Some(yerba::QuoteStyle::Double),
    })
    .unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_enforce_styles_partial_only_collection() {
  use yerba::document::style::StyleEnforcement;

  let mut document = parse(indoc! {"
    tags: [ruby, rails]
    name: localhost
  "});

  document
    .enforce_styles(&StyleEnforcement {
      collection_style: Some("block".to_string()),
      sequence_indent: None,
      key_style: None,
      value_style: None,
    })
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - rails
      name: localhost
    "}
  );
}
