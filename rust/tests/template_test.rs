mod support;
use indoc::indoc;
use std::collections::HashMap;
use support::parse;
use yerba::{resolve_template, Variable};

#[test]
fn test_template_no_references() {
  let document = parse("id: test\n");
  let variables = HashMap::new();

  let result = resolve_template("plain text", &document, None, &variables).unwrap();

  assert_eq!(result, "plain text");
}

#[test]
fn test_template_simple_reference() {
  let document = parse("id: my-talk\n");
  let variables = HashMap::new();

  let result = resolve_template("${id}", &document, None, &variables).unwrap();

  assert_eq!(result, "my-talk");
}

#[test]
fn test_template_mixed_literal_and_reference() {
  let document = parse("id: my-talk\n");
  let variables = HashMap::new();

  let result = resolve_template("https://example.com/${id}", &document, None, &variables).unwrap();

  assert_eq!(result, "https://example.com/my-talk");
}

#[test]
fn test_template_multiple_references() {
  let document = parse(indoc! {"
    first: hello
    last: world
  "});
  let variables = HashMap::new();

  let result = resolve_template("${first}-${last}", &document, None, &variables).unwrap();

  assert_eq!(result, "hello-world");
}

#[test]
fn test_template_nested_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
  "});
  let variables = HashMap::new();

  let result = resolve_template("${database.host}", &document, None, &variables).unwrap();

  assert_eq!(result, "localhost");
}

#[test]
fn test_template_with_index() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});
  let variables = HashMap::new();

  let result = resolve_template("${tags[0]}", &document, None, &variables).unwrap();

  assert_eq!(result, "ruby");
}

#[test]
fn test_template_missing_reference_errors() {
  let document = parse("id: test\n");
  let variables = HashMap::new();

  let result = resolve_template("${missing}", &document, None, &variables);

  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("missing"));
}

#[test]
fn test_template_unclosed_bracket_errors() {
  let document = parse("id: test\n");
  let variables = HashMap::new();

  let result = resolve_template("${unclosed", &document, None, &variables);

  assert!(result.is_err());
}

#[test]
fn test_template_resolves_single_variable() {
  let document = parse("id: test\n");
  let mut variables = HashMap::new();

  variables.insert("my_var".to_string(), Variable::Single("stored_value".to_string()));

  let result = resolve_template("${my_var}", &document, None, &variables).unwrap();

  assert_eq!(result, "stored_value");
}

#[test]
fn test_template_resolves_list_variable() {
  let document = parse("id: test\n");
  let mut variables = HashMap::new();

  variables.insert(
    "names".to_string(),
    Variable::List(vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()]),
  );

  let result = resolve_template("${names}", &document, None, &variables).unwrap();

  assert_eq!(result, "Alice, Bob, Charlie");
}

#[test]
fn test_template_variable_takes_precedence_over_document() {
  let document = parse("id: from-document\n");
  let mut variables = HashMap::new();

  variables.insert("id".to_string(), Variable::Single("from-variable".to_string()));

  let result = resolve_template("${id}", &document, None, &variables).unwrap();

  assert_eq!(result, "from-variable");
}

#[test]
fn test_template_with_base_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});
  let variables = HashMap::new();

  let result = resolve_template("${host}", &document, Some("database"), &variables).unwrap();

  assert_eq!(result, "localhost");
}

#[test]
fn test_simulated_get_then_set_pipeline() {
  let mut document = parse(indoc! {"
    id: my-talk
    slug: placeholder
  "});
  let mut variables = HashMap::new();

  let value = document.get("id").unwrap();
  variables.insert("video_id".to_string(), Variable::Single(value));

  let resolved = resolve_template("${video_id}", &document, None, &variables).unwrap();
  document.set("slug", &resolved).unwrap();

  assert_eq!(document.get("slug"), Some("my-talk".to_string()));
}

#[test]
fn test_simulated_get_then_set_with_prefix() {
  let mut document = parse(indoc! {"
    id: my-talk
    url: placeholder
  "});
  let mut variables = HashMap::new();

  let value = document.get("id").unwrap();
  variables.insert("id".to_string(), Variable::Single(value));

  let resolved = resolve_template("https://example.com/talks/${id}", &document, None, &variables).unwrap();
  document.set("url", &resolved).unwrap();

  assert_eq!(
    document.get("url"),
    Some("https://example.com/talks/my-talk".to_string())
  );
}

#[test]
fn test_template_resolves_after_mutation() {
  let mut document = parse(indoc! {"
    name: old
    title: placeholder
  "});
  let variables = HashMap::new();

  document.set("name", "new").unwrap();

  let result = resolve_template("${name}", &document, None, &variables).unwrap();

  assert_eq!(result, "new");
}
