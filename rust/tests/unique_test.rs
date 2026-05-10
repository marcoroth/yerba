mod support;
use indoc::indoc;
use support::parse;

fn values(duplicates: &[yerba::DuplicateInfo]) -> Vec<&str> {
  duplicates.iter().map(|duplicate| duplicate.value.as_str()).collect()
}

#[test]
fn test_unique_finds_duplicates() {
  let mut document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
    - id: a
      title: Duplicate
  "});

  let duplicates = document.unique("", ".id", false).unwrap();

  assert_eq!(values(&duplicates), vec!["a"]);
}

#[test]
fn test_unique_no_duplicates() {
  let mut document = parse(indoc! {"
    - id: a
    - id: b
    - id: c
  "});

  let duplicates = document.unique("", ".id", false).unwrap();

  assert!(duplicates.is_empty());
}

#[test]
fn test_unique_removes_duplicates() {
  let mut document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
    - id: a
      title: Duplicate
  "});

  let duplicates = document.unique("", ".id", true).unwrap();

  assert_eq!(values(&duplicates), vec!["a"]);
  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        title: First
      - id: b
        title: Second
    "}
  );
}

#[test]
fn test_unique_scalar_array() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - ruby
      - yaml
  "});

  let duplicates = document.unique("tags", ".", true).unwrap();

  assert_eq!(values(&duplicates), vec!["ruby"]);
  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - rust
        - yaml
    "}
  );
}

#[test]
fn test_unique_multiple_duplicates() {
  let mut document = parse(indoc! {"
    - id: a
    - id: b
    - id: a
    - id: c
    - id: b
  "});

  let duplicates = document.unique("", ".id", true).unwrap();

  assert_eq!(values(&duplicates), vec!["a", "b"]);
  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
      - id: b
      - id: c
    "}
  );
}

#[test]
fn test_unique_keeps_first_occurrence() {
  let mut document = parse(indoc! {"
    - id: a
      title: First
    - id: a
      title: Second
    - id: a
      title: Third
  "});

  document.unique("", ".id", true).unwrap();

  assert_eq!(document.get("[0].title"), Some("First".to_string()));
  assert_eq!(document.get_all("[].id").len(), 1);
}

#[test]
fn test_unique_nested_sequence() {
  let mut document = parse(indoc! {"
    speakers:
      - name: Alice
      - name: Bob
      - name: Alice
  "});

  let duplicates = document.unique("speakers", ".name", true).unwrap();

  assert_eq!(values(&duplicates), vec!["Alice"]);
  assert_eq!(
    document.to_string(),
    indoc! {"
      speakers:
        - name: Alice
        - name: Bob
    "}
  );
}

#[test]
fn test_unique_without_remove_does_not_modify() {
  let mut document = parse(indoc! {"
    - id: a
    - id: a
  "});

  let original = document.to_string();
  let duplicates = document.unique("", ".id", false).unwrap();

  assert_eq!(values(&duplicates), vec!["a"]);
  assert_eq!(document.to_string(), original);
}

#[test]
fn test_unique_single_item() {
  let mut document = parse(indoc! {"
    - id: a
  "});

  let duplicates = document.unique("", ".id", false).unwrap();

  assert!(duplicates.is_empty());
}

#[test]
fn test_unique_blank_values_are_duplicates_by_default() {
  let mut document = parse(indoc! {"
    - name: Alice
      github: \"\"
    - name: Bob
      github: \"\"
  "});

  let duplicates = document.unique("", ".github", false).unwrap();

  assert_eq!(values(&duplicates), vec![""]);
}

#[test]
fn test_unique_allow_blank_duplicates_skips_empty() {
  let mut document = parse(indoc! {"
    - name: Alice
      github: \"\"
    - name: Bob
      github: \"\"
  "});

  let duplicates = document.unique_with_options("", ".github", false, true).unwrap();

  assert!(duplicates.is_empty());
}

#[test]
fn test_unique_allow_blank_duplicates_still_finds_non_blank() {
  let mut document = parse(indoc! {"
    - name: Alice
      github: \"alice\"
    - name: Bob
      github: \"\"
    - name: Charlie
      github: \"alice\"
  "});

  let duplicates = document.unique_with_options("", ".github", false, true).unwrap();

  assert_eq!(values(&duplicates), vec!["alice"]);
}

#[test]
fn test_unique_missing_field_skipped() {
  let mut document = parse(indoc! {"
    - name: Alice
    - name: Bob
  "});

  let duplicates = document.unique("", ".github", false).unwrap();

  assert!(duplicates.is_empty());
}

#[test]
fn test_unique_missing_field_always_skipped_even_without_allow_blank() {
  let mut document = parse(indoc! {"
    - name: Alice
    - name: Bob
  "});

  let duplicates = document.unique_with_options("", ".github", false, false).unwrap();

  assert!(duplicates.is_empty());
}

#[test]
fn test_unique_mixed_missing_and_present() {
  let mut document = parse(indoc! {r#"
    - name: Alice
      github: "alice"
    - name: Bob
    - name: Charlie
      github: "alice"
  "#});

  let duplicates = document.unique("", ".github", false).unwrap();

  assert_eq!(values(&duplicates), vec!["alice"]);
}

#[test]
fn test_unique_missing_vs_empty_are_different() {
  let mut document = parse(indoc! {r#"
    - name: Alice
      github: ""
    - name: Bob
    - name: Charlie
      github: ""
  "#});

  let duplicates = document.unique("", ".github", false).unwrap();

  assert_eq!(values(&duplicates), vec![""]);
}

#[test]
fn test_unique_remove_blank_duplicates() {
  let mut document = parse(indoc! {"
    - name: Alice
      github: \"\"
    - name: Bob
      github: \"bob\"
    - name: Charlie
      github: \"\"
  "});

  let duplicates = document.unique("", ".github", true).unwrap();

  assert_eq!(values(&duplicates), vec![""]);
  assert_eq!(document.get_all("[].name"), vec!["Alice", "Bob"]);
}

#[test]
fn test_unique_scalar_array_with_blanks() {
  let mut document = parse(indoc! {r#"
    tags:
      - ruby
      - ""
      - rust
      - ""
      - ruby
  "#});

  let duplicates = document.unique("tags", ".", true).unwrap();

  assert_eq!(values(&duplicates), vec!["", "ruby"]);
  assert_eq!(
    document.to_string(),
    indoc! {r#"
      tags:
        - ruby
        - ""
        - rust
    "#}
  );
}

#[test]
fn test_unique_scalar_array_with_blanks_allow() {
  let mut document = parse(indoc! {r#"
    tags:
      - ruby
      - ""
      - rust
      - ""
      - ruby
  "#});

  let duplicates = document.unique_with_options("tags", ".", true, true).unwrap();

  assert_eq!(values(&duplicates), vec!["ruby"]);
  assert_eq!(
    document.to_string(),
    indoc! {r#"
      tags:
        - ruby
        - ""
        - rust
        - ""
    "#}
  );
}
