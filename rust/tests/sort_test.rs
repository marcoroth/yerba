mod support;
use indoc::indoc;
use support::parse;
use yerba::SortField;

#[test]
fn test_sort_simple_array() {
  let mut document = parse(indoc! {"
    tags:
      - rust
      - yaml
      - ruby
  "});

  document.sort_items("tags", &[], true).unwrap();

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
fn test_sort_already_sorted_noop() {
  let mut document = parse(indoc! {"
    tags:
      - a
      - b
      - c
  "});

  document.sort_items("tags", &[], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - a
        - b
        - c
    "}
  );
}

#[test]
fn test_sort_by_field() {
  let mut document = parse(indoc! {"
    - name: Charlie
    - name: Alice
    - name: Bob
  "});

  document.sort_items("", &[SortField::asc("name")], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
      - name: Bob
      - name: Charlie
    "}
  );
}

#[test]
fn test_sort_by_field_descending() {
  let mut document = parse(indoc! {"
    - name: Alice
    - name: Bob
    - name: Charlie
  "});

  document.sort_items("", &[SortField::desc("name")], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Charlie
      - name: Bob
      - name: Alice
    "}
  );
}

#[test]
fn test_sort_by_multiple_fields() {
  let mut document = parse(indoc! {"
    - kind: talk
      title: Zebra
    - kind: keynote
      title: Alpha
    - kind: talk
      title: Alpha
  "});

  document
    .sort_items("", &[SortField::asc("kind"), SortField::asc("title")], true)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - kind: keynote
        title: Alpha
      - kind: talk
        title: Alpha
      - kind: talk
        title: Zebra
    "}
  );
}

#[test]
fn test_sort_by_mixed_directions() {
  let mut document = parse(indoc! {"
    - kind: talk
      title: B
    - kind: keynote
      title: A
    - kind: talk
      title: A
  "});

  document
    .sort_items("", &[SortField::asc("kind"), SortField::desc("title")], true)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - kind: keynote
        title: A
      - kind: talk
        title: B
      - kind: talk
        title: A
    "}
  );
}

#[test]
fn test_sort_nested_sequence() {
  let mut document = parse(indoc! {"
    data:
      items:
        - c
        - a
        - b
  "});

  document.sort_items("data.items", &[], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      data:
        items:
          - a
          - b
          - c
    "}
  );
}

#[test]
fn test_sort_each_nested_sequence() {
  let mut document = parse(indoc! {"
    - tags:
        - rust
        - ruby
    - tags:
        - yaml
        - go
  "});

  document.sort_items("[].tags", &[], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - tags:
          - ruby
          - rust
      - tags:
          - go
          - yaml
    "}
  );
}

#[test]
fn test_sort_single_item_noop() {
  let mut document = parse(indoc! {"
    tags:
      - only
  "});

  document.sort_items("tags", &[], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - only
    "}
  );
}

#[test]
fn test_sort_field_parse() {
  let field = SortField::parse("title");
  assert_eq!(field.path, "title");
  assert!(field.ascending);

  let field = SortField::parse("date:desc");
  assert_eq!(field.path, "date");
  assert!(!field.ascending);
}

#[test]
fn test_sort_field_parse_list() {
  let fields = SortField::parse_list("kind,date:desc,title");

  assert_eq!(fields.len(), 3);
  assert_eq!(fields[0].path, "kind");
  assert!(fields[0].ascending);
  assert_eq!(fields[1].path, "date");
  assert!(!fields[1].ascending);
  assert_eq!(fields[2].path, "title");
  assert!(fields[2].ascending);
}

#[test]
fn test_sort_preserves_comments() {
  let mut document = parse(indoc! {"
    # tags
    tags:
      - rust
      - ruby
    # end
  "});

  document.sort_items("tags", &[], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # tags
      tags:
        - ruby
        - rust
      # end
    "}
  );
}

#[test]
fn test_sort_case_insensitive() {
  let mut document = parse(indoc! {"
    - name: charlie
    - name: Alice
    - name: Bob
  "});

  document.sort_items("", &[SortField::asc("name")], false).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
      - name: Bob
      - name: charlie
    "}
  );
}

#[test]
fn test_sort_case_sensitive() {
  let mut document = parse(indoc! {"
    - name: charlie
    - name: Alice
    - name: Bob
  "});

  document.sort_items("", &[SortField::asc("name")], true).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
      - name: Bob
      - name: charlie
    "}
  );
}

#[test]
fn test_sort_case_insensitive_simple_array() {
  let mut document = parse(indoc! {"
    tags:
      - Rust
      - apple
      - Banana
  "});

  document.sort_items("tags", &[], false).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - apple
        - Banana
        - Rust
    "}
  );
}

#[test]
fn test_sort_preserves_preceding_section_comments() {
  let mut document = parse(indoc! {"
    ## Section B
    - name: Charlie
      slug: charlie

    ## Section A
    - name: Alice
      slug: alice
  "});

  document.sort_items("", &[SortField::asc("name")], false).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      ## Section A
      - name: Alice
        slug: alice

      ## Section B
      - name: Charlie
        slug: charlie
    "}
  );
}

#[test]
fn test_sort_preserves_trailing_comments_on_entries() {
  let mut document = parse(indoc! {"
    - name: Charlie
      slug: charlie
      # extra info
    - name: Alice
      slug: alice
  "});

  document.sort_items("", &[SortField::asc("name")], false).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
        slug: alice
      - name: Charlie
        slug: charlie
        # extra info
    "}
  );
}

#[test]
fn test_sort_each_preserves_comments() {
  let mut document = parse(indoc! {"
    - id: talk-1
      speakers:
        - Charlie # important
        - Alice
  "});

  document.sort_items("[].speakers", &[], false).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        speakers:
          - Alice
          - Charlie # important
    "}
  );
}

#[test]
fn test_sort_preserves_multiple_section_headers() {
  let mut document = parse(indoc! {"
    ## Day 2

    - id: talk-b
      title: B

    ## Day 1

    - id: talk-a
      title: A
  "});

  document.sort_items("", &[SortField::asc("title")], false).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      ## Day 1
      - id: talk-a
        title: A

      ## Day 2
      - id: talk-b
        title: B
    "}
  );
}
