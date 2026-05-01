use std::collections::HashMap;
use yerba::{resolve_template, Document, Variable};

#[test]
fn test_template_no_references() {
  let document = Document::parse("id: test\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("plain text", &document, None, &variables).unwrap();

  assert_eq!(result, "plain text");
}

#[test]
fn test_template_simple_reference() {
  let document = Document::parse("id: my-talk\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("${id}", &document, None, &variables).unwrap();

  assert_eq!(result, "my-talk");
}

#[test]
fn test_template_mixed_literal_and_reference() {
  let document = Document::parse("id: my-talk\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("https://example.com/${id}", &document, None, &variables).unwrap();

  assert_eq!(result, "https://example.com/my-talk");
}

#[test]
fn test_template_multiple_references() {
  let document = Document::parse("first: hello\nlast: world\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("${first}-${last}", &document, None, &variables).unwrap();

  assert_eq!(result, "hello-world");
}

#[test]
fn test_template_nested_path() {
  let document = Document::parse("database:\n  host: localhost\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("${database.host}", &document, None, &variables).unwrap();

  assert_eq!(result, "localhost");
}

#[test]
fn test_template_with_index() {
  let document = Document::parse("tags:\n  - ruby\n  - rust\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("${tags[0]}", &document, None, &variables).unwrap();

  assert_eq!(result, "ruby");
}

#[test]
fn test_template_missing_reference_errors() {
  let document = Document::parse("id: test\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("${missing}", &document, None, &variables);

  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("missing"));
}

#[test]
fn test_template_unclosed_bracket_errors() {
  let document = Document::parse("id: test\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("${unclosed", &document, None, &variables);

  assert!(result.is_err());
}

#[test]
fn test_template_resolves_single_variable() {
  let document = Document::parse("id: test\n").unwrap();
  let mut variables = HashMap::new();

  variables.insert("my_var".to_string(), Variable::Single("stored_value".to_string()));

  let result = resolve_template("${my_var}", &document, None, &variables).unwrap();

  assert_eq!(result, "stored_value");
}

#[test]
fn test_template_resolves_list_variable() {
  let document = Document::parse("id: test\n").unwrap();
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
  let document = Document::parse("id: from-document\n").unwrap();
  let mut variables = HashMap::new();

  variables.insert("id".to_string(), Variable::Single("from-variable".to_string()));

  let result = resolve_template("${id}", &document, None, &variables).unwrap();

  assert_eq!(result, "from-variable");
}

#[test]
fn test_template_with_base_path() {
  let document = Document::parse("database:\n  host: localhost\n  port: 5432\n").unwrap();
  let variables = HashMap::new();

  let result = resolve_template("${host}", &document, Some("database"), &variables).unwrap();

  assert_eq!(result, "localhost");
}

#[test]
fn test_simulated_get_then_set_pipeline() {
  let yaml = "id: my-talk\nslug: placeholder\n";
  let mut document = Document::parse(yaml).unwrap();
  let mut variables = HashMap::new();

  let value = document.get("id").unwrap();
  variables.insert("video_id".to_string(), Variable::Single(value));

  let resolved = resolve_template("${video_id}", &document, None, &variables).unwrap();
  document.set("slug", &resolved).unwrap();

  assert_eq!(document.get("slug"), Some("my-talk".to_string()));
}

#[test]
fn test_simulated_get_then_set_with_prefix() {
  let yaml = "id: my-talk\nurl: placeholder\n";
  let mut document = Document::parse(yaml).unwrap();
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
  let yaml = "name: old\ntitle: placeholder\n";
  let mut document = Document::parse(yaml).unwrap();
  let variables = HashMap::new();

  document.set("name", "new").unwrap();

  let result = resolve_template("${name}", &document, None, &variables).unwrap();

  assert_eq!(result, "new");
}
