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
