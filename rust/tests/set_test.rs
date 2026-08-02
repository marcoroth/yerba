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

#[test]
fn test_set_multiline_on_block_scalar() {
  let mut document = parse(indoc! {"
    - id: talk-1
      description: |-
        Old text
  "});

  document.set("[0].description", "First line.\nSecond line.").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        description: |-
          First line.
          Second line.
    "}
  );
}

#[test]
fn test_set_multiline_with_blank_line_on_block_scalar() {
  let mut document = parse(indoc! {"
    - id: talk-1
      description: |-
        Old text
  "});

  document.set("[0].description", "First paragraph.\n\nSecond paragraph.").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        description: |-
          First paragraph.

          Second paragraph.
    "}
  );
}

#[test]
fn test_set_multiline_on_nested_block_scalar() {
  let mut document = parse(indoc! {"
    app:
      config:
        description: |-
          Old text
  "});

  document.set("app.config.description", "Line 1.\nLine 2.").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        config:
          description: |-
            Line 1.
            Line 2.
    "}
  );
}

#[test]
fn test_set_multiline_on_top_level_block_scalar() {
  let mut document = parse(indoc! {"
    description: |-
      Old text
  "});

  document.set("description", "New first.\nNew second.").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        New first.
        New second.
    "}
  );
}

#[test]
fn test_set_empty_on_block_scalar() {
  let mut document = parse(indoc! {"
    - id: talk-1
      description: |-
        Old text
  "});

  document.set("[0].description", "").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: talk-1
        description: ""
    "#}
  );
}

#[test]
fn test_set_single_line_on_block_scalar() {
  let mut document = parse(indoc! {"
    - id: talk-1
      description: |-
        Old text
  "});

  document.set("[0].description", "New single line").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: talk-1
        description: "New single line"
    "#}
  );
}

#[test]
fn test_set_all_multiline_on_block_scalars() {
  let mut document = parse(indoc! {"
    - id: talk-1
      description: |-
        Old 1
    - id: talk-2
      description: |-
        Old 2
  "});

  document.set_all("[].description", "Line A.\nLine B.").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        description: |-
          Line A.
          Line B.
      - id: talk-2
        description: |-
          Line A.
          Line B.
    "}
  );
}

fn apps() -> String {
  indoc! {r#"
    apps:
      - name: "alpha"
        license: "MIT"
        license_file: "LICENSE"
      - name: "helios"
        license: "unknown"
        license_file: "README.md"
      - name: "zeta"
        license: "unknown"
        license_file: null
  "#}
  .to_string()
}

#[test]
fn test_set_where_touches_only_the_matching_item() {
  let mut document = parse(&apps());

  let count = document.set_where("apps[]", "license", "AGPL-3.0", ".name == \"helios\"", false).unwrap();

  assert_eq!(count, 1);
  assert_eq!(document.get_all("apps[].license"), vec!["MIT", "AGPL-3.0", "unknown"]);
}

#[test]
fn test_set_where_without_all_errors_on_multiple_matches() {
  let mut document = parse(&apps());

  let result = document.set_where("apps[]", "license", "MIT", ".license == \"unknown\"", false);

  assert!(result.is_err(), "two items match, so this is ambiguous without --all");
  assert_eq!(
    document.get_all("apps[].license"),
    vec!["MIT", "unknown", "unknown"],
    "a rejected set must not edit anything"
  );
}

#[test]
fn test_set_where_with_all_sets_every_match_and_nothing_else() {
  let mut document = parse(&apps());

  let count = document.set_where("apps[]", "license", "MIT", ".license == \"unknown\"", true).unwrap();

  assert_eq!(count, 2);
  assert_eq!(document.get_all("apps[].license"), vec!["MIT", "MIT", "MIT"]);
}

#[test]
fn test_set_where_no_match_is_a_no_op() {
  let mut document = parse(&apps());
  let before = document.to_string();

  let count = document.set_where("apps[]", "license", "MIT", ".name == \"absent\"", true).unwrap();

  assert_eq!(count, 0);
  assert_eq!(document.to_string(), before);
}

#[test]
fn test_set_where_rejects_an_absolute_condition() {
  let mut document = parse(&apps());

  let result = document.set_where("apps[]", "license", "MIT", "name == \"helios\"", true);

  assert!(result.is_err(), "conditions are tested per item, so they must be relative");
}
