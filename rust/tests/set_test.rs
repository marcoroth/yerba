mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_set_plain_scalar() {
  let mut document = parse("host: localhost");

  document.set("host", "0.0.0.0").unwrap();

  assert_eq!(document.to_string(), "host: 0.0.0.0");
}

#[test]
fn test_set_preserves_double_quotes() {
  let mut document = parse("name: \"myapp\"");

  document.set("name", "newapp").unwrap();

  assert_eq!(document.to_string(), "name: \"newapp\"");
}

#[test]
fn test_set_preserves_single_quotes() {
  let mut document = parse("name: 'myapp'");

  document.set("name", "newapp").unwrap();

  assert_eq!(document.to_string(), "name: 'newapp'");
}

#[test]
fn test_set_nested_path() {
  let yaml = indoc! {"
    database:
      host: localhost
      port: 5432
  "};
  let mut document = parse(yaml);

  document.set("database.host", "0.0.0.0").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: 0.0.0.0
        port: 5432
    "}
  );
}

#[test]
fn test_set_preserves_comments() {
  let yaml = indoc! {"
    # Database config
    host: localhost
    # Port
    port: 5432
  "};
  let mut document = parse(yaml);

  document.set("host", "0.0.0.0").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Database config
      host: 0.0.0.0
      # Port
      port: 5432
    "}
  );
}

#[test]
fn test_set_escapes_double_quotes_in_double_quoted_field() {
  let mut document = parse("title: \"old title\"\n");

  document.set("title", "something \"quoted\" here").unwrap();

  assert_eq!(document.to_string(), "title: \"something \\\"quoted\\\" here\"\n");
}

#[test]
fn test_set_escapes_single_quotes_in_single_quoted_field() {
  let mut document = parse("title: 'old title'\n");

  document.set("title", "it's a test").unwrap();

  assert_eq!(document.to_string(), "title: 'it''s a test'\n");
}

#[test]
fn test_set_plain_field_with_value_containing_quotes() {
  let mut document = parse("title: old\n");

  document.set("title", "something \"quoted\"").unwrap();

  assert_eq!(document.to_string(), "title: something \"quoted\"\n");
}

#[test]
fn test_set_with_bracket_index_path() {
  let yaml = indoc! {"
    - id: first
      title: A
    - id: second
      title: B
  "};
  let mut document = parse(yaml);

  document.set("[1].title", "Updated").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        title: A
      - id: second
        title: Updated
    "}
  );
}

#[test]
fn test_set_with_bracket_index_first_item() {
  let yaml = indoc! {"
    - id: first
      title: A
    - id: second
      title: B
  "};
  let mut document = parse(yaml);

  document.set("[0].title", "Updated").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        title: Updated
      - id: second
        title: B
    "}
  );
}

#[test]
fn test_set_block_scalar_to_empty() {
  let yaml = indoc! {"
    - id: talk-1
      description: |-
        Some long description
        across multiple lines
  "};
  let mut document = parse(yaml);

  document.set("[0].description", "").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        description: \"\"
    "}
  );
}

#[test]
fn test_set_block_scalar_to_new_value() {
  let yaml = indoc! {"
    description: |-
      Old description
  "};
  let mut document = parse(yaml);

  document.set("description", "New value").unwrap();

  assert_eq!(document.to_string(), "description: \"New value\"\n");
}

#[test]
fn test_set_bracket_index_out_of_bounds() {
  let mut document = parse("- id: first\n");

  let result = document.set("[5].id", "test");
  assert!(result.is_err());
}

#[test]
fn test_set_bracket_index_multiple_matches_error() {
  let mut document = parse(indoc! {"
    - id: first
      title: A
    - id: second
      title: B
  "});

  let result = document.set("[].title", "test");

  assert!(result.is_err(), "setting on [] (all items) should error — use a specific index");
}

#[test]
fn test_set_all_updates_all_matching_nodes() {
  let mut document = parse(indoc! {"
    - title: Hello
      description: some text
    - title: World
      description: other text
  "});

  document.set_all("[].description", "").unwrap();

  let output = document.to_string();
  assert!(output.contains("description: \n") || output.contains("description:\n"));
  assert!(!output.contains("some text"));
  assert!(!output.contains("other text"));
}

#[test]
fn test_set_all_preserves_quote_style() {
  let mut document = parse(indoc! {r#"
    - name: "Alice"
    - name: "Bob"
  "#});

  document.set_all("[].name", "Updated").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - name: "Updated"
      - name: "Updated"
    "#}
  );
}

#[test]
fn test_set_all_errors_on_missing_path() {
  let mut document = parse(indoc! {"
    - name: Alice
  "});

  let result = document.set_all("[].missing", "value");
  assert!(result.is_err());
}

#[test]
fn test_set_all_replaces_block_scalars_with_empty_string() {
  let mut document = parse(indoc! {r#"
    - title: "First"
      description: |-
        This is a block scalar
        with multiple lines
    - title: "Second"
      description: |-
        Another block scalar
  "#});

  document.set_all("[].description", "").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - title: "First"
      description: ""
    - title: "Second"
      description: ""
  "#}
  );
}

#[test]
fn test_set_all_replaces_block_scalars_with_value() {
  let mut document = parse(indoc! {r#"
    - title: "First"
      description: |-
        Old description
    - title: "Second"
      description: |-
        Another old one
  "#});

  document.set_all("[].description", "Updated").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - title: "First"
      description: "Updated"
    - title: "Second"
      description: "Updated"
  "#}
  );
}

#[test]
fn test_set_all_handles_mixed_block_and_inline_scalars() {
  let mut document = parse(indoc! {r#"
    - title: "First"
      description: |-
        Block scalar here
    - title: "Second"
      description: "Inline scalar"
  "#});

  document.set_all("[].description", "").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - title: "First"
      description: ""
    - title: "Second"
      description: ""
  "#}
  );
}
