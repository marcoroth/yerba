use indoc::indoc;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_from_discovers_yerbafile_in_same_directory() {
  let dir = TempDir::new().unwrap();
  fs::write(dir.path().join("Yerbafile"), "rules: []").unwrap();

  let result = yerba::Yerbafile::find_from(dir.path());

  assert_eq!(result, Some(dir.path().join("Yerbafile")));
}

#[test]
fn test_find_from_discovers_yerbafile_yml() {
  let dir = TempDir::new().unwrap();
  fs::write(dir.path().join("Yerbafile.yml"), "rules: []").unwrap();

  let result = yerba::Yerbafile::find_from(dir.path());

  assert_eq!(result, Some(dir.path().join("Yerbafile.yml")));
}

#[test]
fn test_find_from_discovers_yerbafile_yaml() {
  let dir = TempDir::new().unwrap();
  fs::write(dir.path().join("Yerbafile.yaml"), "rules: []").unwrap();

  let result = yerba::Yerbafile::find_from(dir.path());

  assert_eq!(result, Some(dir.path().join("Yerbafile.yaml")));
}

#[test]
fn test_find_from_discovers_dot_yerbafile() {
  let dir = TempDir::new().unwrap();
  fs::write(dir.path().join(".yerbafile"), "rules: []").unwrap();

  let result = yerba::Yerbafile::find_from(dir.path());

  assert_eq!(result, Some(dir.path().join(".yerbafile")));
}

#[test]
fn test_find_from_prefers_yerbafile_over_yerbafile_yml() {
  let dir = TempDir::new().unwrap();
  fs::write(dir.path().join("Yerbafile"), "rules: []").unwrap();
  fs::write(dir.path().join("Yerbafile.yml"), "rules: []").unwrap();

  let result = yerba::Yerbafile::find_from(dir.path());

  assert_eq!(result, Some(dir.path().join("Yerbafile")));
}

#[test]
fn test_find_from_walks_up_to_parent_directory() {
  let dir = TempDir::new().unwrap();
  let child = dir.path().join("sub").join("deep");
  fs::create_dir_all(&child).unwrap();
  fs::write(dir.path().join("Yerbafile"), "rules: []").unwrap();

  let result = yerba::Yerbafile::find_from(&child);

  assert_eq!(result, Some(dir.path().join("Yerbafile")));
}

#[test]
fn test_find_from_returns_none_when_not_found() {
  let dir = TempDir::new().unwrap();

  let result = yerba::Yerbafile::find_from(dir.path());

  assert_eq!(result, None);
}

#[test]
fn test_load_parses_yerbafile() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - sort_keys:
              path: "[]"
              order:
                - name
                - slug
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  assert_eq!(yerbafile.rules.len(), 1);
  assert_eq!(yerbafile.rules[0].files, "**/*.yml");
}

#[test]
fn test_apply_to_document_reorders_keys() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - sort_keys:
              path: "[]"
              order:
                - name
                - slug
                - github
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    - github: aalice
      name: Alice
      slug: alice
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/speakers.yml").unwrap();

  assert!(changed);
  assert_eq!(
    document.to_string(),
    indoc! {"
    - name: Alice
      slug: alice
      github: aalice
  "}
  );
}

#[test]
fn test_apply_to_document_skips_non_matching_rules() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "data/videos/**/*.yml"
        pipeline:
          - sort_keys:
              path: "[]"
              order:
                - title
                - id
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    - id: talk-1
      title: Hello
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/speakers.yml").unwrap();

  assert!(!changed);
  assert_eq!(
    document.to_string(),
    indoc! {"
    - id: talk-1
      title: Hello
  "}
  );
}

#[test]
fn test_apply_to_document_applies_all_matching_rules() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - quote_style:
              key_style: plain
              value_style: double
          - sort_keys:
              path: "[]"
              order:
                - name
                - slug
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    - slug: alice
      name: Alice
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/speakers.yml").unwrap();

  assert!(changed);
  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - name: "Alice"
      slug: "alice"
  "#}
  );
}

#[test]
fn test_apply_collection_style_flow_to_block() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - collection_style:
              path: "tags"
              style: block
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    tags: [ruby, rails]
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/config.yml").unwrap();

  assert!(changed);
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
fn test_apply_collection_style_block_to_flow() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - collection_style:
              path: "tags"
              style: flow
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    tags:
      - ruby
      - rails
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/config.yml").unwrap();

  assert!(changed);
  assert_eq!(
    document.to_string(),
    indoc! {"
    tags: [ruby, rails]
  "}
  );
}

#[test]
fn test_apply_collection_style_with_wildcard_path() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - collection_style:
              path: "[].tags"
              style: block
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    - name: Alice
      tags: [ruby, rails]
    - name: Bob
      tags: [python, django]
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/speakers.yml").unwrap();

  assert!(changed);
  assert_eq!(
    document.to_string(),
    indoc! {"
    - name: Alice
      tags:
        - ruby
        - rails
    - name: Bob
      tags:
        - python
        - django
  "}
  );
}

#[test]
fn test_apply_collection_style_without_path_applies_to_whole_file() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - collection_style:
              style: block
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    tags: [ruby, rails]
    database: {host: localhost, port: 5432}
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/config.yml").unwrap();

  assert!(changed);
  assert_eq!(
    document.to_string(),
    indoc! {"
    tags:
      - ruby
      - rails
    database:
      host: localhost
      port: 5432
  "}
  );
}

#[test]
fn test_apply_collection_style_without_path_noop_when_already_matching() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - collection_style:
              style: block
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    tags:
      - ruby
      - rails
  "})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/config.yml").unwrap();

  assert!(!changed);
}

#[test]
fn test_schema_validation_passes_for_valid_document() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {r#"
    name: "test"
    port: 5432
  "#})
  .unwrap();

  let result = yerbafile.apply_to_document(&mut document, "config.yml");

  assert!(result.is_ok());
}

#[test]
fn test_schema_validation_fails_for_invalid_document() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {r#"
    port: "not a number"
  "#})
  .unwrap();

  let result = yerbafile.apply_to_document(&mut document, "config.yml");

  assert!(result.is_err());
  let error = result.unwrap_err().to_string();
  assert!(error.contains("schema validation failed"), "error was: {}", error);
  assert!(error.contains("name"), "error should mention missing 'name': {}", error);
}

#[test]
fn test_schema_validation_with_items_validates_each_array_entry() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("speaker_schema.json"),
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "speaker_schema.json"
              items: true
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {r#"
    - name: "Alice"
      slug: "alice"
    - name: "Bob"
  "#})
  .unwrap();

  let result = yerbafile.apply_to_document(&mut document, "speakers.yml");

  assert!(result.is_err());
  let error = result.unwrap_err().to_string();
  assert!(error.contains("Bob"), "error should mention item label: {}", error);
  assert!(error.contains("slug"), "error should mention missing 'slug': {}", error);
}

#[test]
fn test_schema_validation_with_items_passes_for_valid_array() {
  let dir = TempDir::new().unwrap();

  fs::write(
    dir.path().join("speaker_schema.json"),
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "speaker_schema.json"
              items: true
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {r#"
    - name: "Alice"
      slug: "alice"
    - name: "Bob"
      slug: "bob"
  "#})
  .unwrap();

  let result = yerbafile.apply_to_document(&mut document, "speakers.yml");

  assert!(result.is_ok());
}

#[test]
fn test_schema_array_document_fails_against_object_schema_without_path() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    - name: Alice
    - name: Bob
  "})
  .unwrap();

  let result = yerbafile.apply_to_document(&mut document, "test.yml");
  assert!(result.is_err(), "array doc against object schema should fail without path");
}

#[test]
fn test_schema_with_path_validates_each_item() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
              path: "[]"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut valid = yerba::Document::parse(indoc! {"
    - name: Alice
    - name: Bob
  "})
  .unwrap();

  assert!(yerbafile.apply_to_document(&mut valid, "test.yml").is_ok());

  let mut invalid = yerba::Document::parse(indoc! {"
    - name: Alice
    - slug: bob
  "})
  .unwrap();

  let result = yerbafile.apply_to_document(&mut invalid, "test.yml");
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("name"));
}

#[test]
fn test_schema_validation_with_path_selector() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
              path: "tiers[]"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    tiers:
      - name: Gold
        level: 1
      - name: Silver
        level: 2
  "})
  .unwrap();

  assert!(yerbafile.apply_to_document(&mut document, "sponsors.yml").is_ok());
}

#[test]
fn test_schema_validation_with_path_selector_fails_on_invalid() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
              path: "tiers[]"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {"
    tiers:
      - name: Gold
      - name: Silver
        level: 2
  "})
  .unwrap();

  let result = yerbafile.apply_to_document(&mut document, "sponsors.yml");
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("level"));
}

#[test]
fn test_schema_validation_empty_array_passes_with_path() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
              path: "[]"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse("---\n[]\n").unwrap();

  assert!(yerbafile.apply_to_document(&mut document, "test.yml").is_ok());
}

#[test]
fn test_schema_validation_null_document_errors_for_array_schema() {
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

  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - schema:
              file: "schema.json"
              items: true
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse("---\n").unwrap();

  let result = yerbafile.apply_to_document(&mut document, "test.yml");
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("expected an array"));
}

#[test]
fn test_schema_validation_reports_line_numbers() {
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

  let document = yerba::Document::parse(indoc! {"
    - name: Alice
      slug: alice
    - name: Bob
    - name: Charlie
      slug: charlie
  "})
  .unwrap();

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert_eq!(errors.len(), 1);
  assert!(errors[0].line.is_some());
  assert!(errors[0].item_label.as_deref() == Some("Bob"));
}

#[test]
fn test_schema_validation_additional_properties_false() {
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

  let document = yerba::Document::parse(indoc! {"
    - name: Alice
      extra: bad
  "})
  .unwrap();

  let schema = yerba::schema::load_schema(dir.path().join("schema.json")).unwrap();
  let errors = document.validate_schema(&schema, false, Some("[]"));

  assert_eq!(errors.len(), 1);
  assert!(errors[0].message.contains("extra"), "error was: {}", errors[0].message);
}

#[test]
fn test_delete_with_wildcard_and_condition() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - delete:
              path: "[].website"
              condition: '.website == ""'
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {r#"
    - name: "Speaker 1"
      website: "https://speaker1.com"
      slug: "speaker-1"
    - name: "Speaker 2"
      website: ""
      slug: "speaker-2"
    - name: "Speaker 3"
      website: ""
      slug: "speaker-3"
  "#})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/speakers.yml").unwrap();

  assert!(changed);
  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - name: "Speaker 1"
      website: "https://speaker1.com"
      slug: "speaker-1"
    - name: "Speaker 2"
      slug: "speaker-2"
    - name: "Speaker 3"
      slug: "speaker-3"
  "#}
  );
}

#[test]
fn test_delete_with_wildcard_without_condition() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - delete:
              path: "[].website"
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {r#"
    - name: "Speaker 1"
      website: "https://speaker1.com"
    - name: "Speaker 2"
    - name: "Speaker 3"
      website: ""
  "#})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/speakers.yml").unwrap();

  assert!(changed);
  assert_eq!(
    document.to_string(),
    indoc! {r#"
    - name: "Speaker 1"
    - name: "Speaker 2"
    - name: "Speaker 3"
  "#}
  );
}

#[test]
fn test_delete_with_wildcard_no_matches() {
  let dir = TempDir::new().unwrap();
  fs::write(
    dir.path().join("Yerbafile"),
    indoc! {r#"
    rules:
      - files: "**/*.yml"
        pipeline:
          - delete:
              path: "[].website"
              condition: '.website == ""'
  "#},
  )
  .unwrap();

  let yerbafile = yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap();

  let mut document = yerba::Document::parse(indoc! {r#"
    - name: "Speaker 1"
      website: "https://speaker1.com"
    - name: "Speaker 2"
      website: "https://speaker2.com"
  "#})
  .unwrap();

  let changed = yerbafile.apply_to_document(&mut document, "data/speakers.yml").unwrap();

  assert!(!changed);
}
