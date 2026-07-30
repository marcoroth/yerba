#![cfg(feature = "schema")]

use indoc::indoc;
use std::fs;
use tempfile::TempDir;

mod support;
use support::parse;

#[test]
fn test_validate_schema_valid_object() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "port": { "type": "integer" }
      },
      "required": ["name"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    name: test
    port: 5432
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, None);

  assert!(errors.is_empty());
}

#[test]
fn test_validate_schema_missing_required_field() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" }
      },
      "required": ["name"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    port: 5432
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, None);

  assert_eq!(errors.len(), 1);
  assert!(errors[0].message.contains("name"), "error was: {}", errors[0].message);
}

#[test]
fn test_validate_schema_wrong_type() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "port": { "type": "integer" }
      }
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {r#"
    port: "not a number"
  "#});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, None);

  assert_eq!(errors.len(), 1);
  assert!(errors[0].message.contains("integer"), "error was: {}", errors[0].message);
}

#[test]
fn test_validate_schema_additional_properties_false() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" }
      },
      "additionalProperties": false
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    name: Alice
    extra: bad
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, None);

  assert_eq!(errors.len(), 1);
  assert!(errors[0].message.contains("extra"), "error was: {}", errors[0].message);
}

#[test]
fn test_validate_schema_with_selector_validates_each_item() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "slug": { "type": "string" }
      },
      "required": ["name", "slug"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    - name: Alice
      slug: alice
    - name: Bob
    - name: Charlie
      slug: charlie
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert_eq!(errors.len(), 1);
  assert!(errors[0].message.contains("slug"), "error was: {}", errors[0].message);
  assert_eq!(errors[0].item_label.as_deref(), Some("Bob"));
}

#[test]
fn test_validate_schema_with_selector_all_valid() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" }
      },
      "required": ["name"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    - name: Alice
    - name: Bob
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert!(errors.is_empty());
}

#[test]
fn test_validate_schema_empty_array_with_selector() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" }
      },
      "required": ["name"]
    }
  "#},
  )
  .unwrap();

  let document = parse("---\n[]\n");

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert!(errors.is_empty());
}

#[test]
fn test_validate_schema_null_document_with_items_true() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" }
      }
    }
  "#},
  )
  .unwrap();

  let document = parse("---\n");

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, true, None);

  assert_eq!(errors.len(), 1);
  assert!(errors[0].message.contains("expected an array"), "error was: {}", errors[0].message);
}

#[test]
fn test_validate_schema_null_document_without_items() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" }
      }
    }
  "#},
  )
  .unwrap();

  let document = parse("---\n");

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, None);

  assert!(errors.is_empty());
}

#[test]
fn test_validate_schema_reports_line_numbers() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "slug": { "type": "string" }
      },
      "required": ["name", "slug"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    - name: Alice
      slug: alice
    - name: Bob
    - name: Charlie
      slug: charlie
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert_eq!(errors.len(), 1);
  assert!(errors[0].line.is_some());
  assert_eq!(errors[0].line.unwrap(), 3);
}

#[test]
fn test_validate_schema_nested_path() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "level": { "type": "integer" }
      },
      "required": ["name", "level"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    tiers:
      - name: Gold
        level: 1
      - name: Silver
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("tiers[]"));

  assert_eq!(errors.len(), 1);
  assert!(errors[0].message.contains("level"), "error was: {}", errors[0].message);
  assert_eq!(errors[0].item_label.as_deref(), Some("Silver"));
}

#[test]
fn test_validate_schema_enum_values() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "kind": {
          "type": "string",
          "enum": ["conference", "meetup", "workshop"]
        }
      }
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    kind: hackathon
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, None);

  assert_eq!(errors.len(), 1);
  assert!(
    errors[0].message.contains("hackathon") || errors[0].message.contains("enum"),
    "error was: {}",
    errors[0].message
  );
}

#[test]
fn test_validate_schema_single_item_array_with_selector() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" }
      },
      "required": ["name"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    - name: Alice
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert!(errors.is_empty());
}

#[test]
fn test_validate_schema_multiple_errors_per_item() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("schema.json"),
    indoc! {r#"
    {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "slug": { "type": "string" },
        "github": { "type": "string" }
      },
      "required": ["name", "slug", "github"]
    }
  "#},
  )
  .unwrap();

  let document = parse(indoc! {"
    - title: oops
  "});

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert!(errors.len() >= 3, "expected at least 3 errors, got {}", errors.len());
}
