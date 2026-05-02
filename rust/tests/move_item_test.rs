mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_move_item_by_index() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  document.move_item("tags", 2, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - yaml
        - ruby
        - rust
    "}
  );
}

#[test]
fn test_move_item_forward() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  document.move_item("tags", 0, 2).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - rust
        - yaml
        - ruby
    "}
  );
}

#[test]
fn test_move_item_same_index() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.move_item("tags", 0, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - rust
    "}
  );
}

#[test]
fn test_move_item_out_of_bounds() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  assert!(document.move_item("tags", 5, 0).is_err());
}

#[test]
fn test_move_item_preserves_comments() {
  let mut document = parse(indoc! {"
    # Tags
    tags:
      - ruby
      - rust
    # End
  "});

  document.move_item("tags", 1, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Tags
      tags:
        - rust
        - ruby
      # End
    "}
  );
}

#[test]
fn test_resolve_sequence_index_by_name() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  assert_eq!(document.resolve_sequence_index("tags", "ruby").unwrap(), 0);
  assert_eq!(document.resolve_sequence_index("tags", "rust").unwrap(), 1);
  assert_eq!(document.resolve_sequence_index("tags", "yaml").unwrap(), 2);
}

#[test]
fn test_resolve_sequence_index_by_number() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  assert_eq!(document.resolve_sequence_index("tags", "0").unwrap(), 0);
  assert_eq!(document.resolve_sequence_index("tags", "1").unwrap(), 1);
}

#[test]
fn test_move_item_preserves_inline_comment() {
  let mut document = parse(indoc! {"
    - name: Charlie # VIP
      slug: charlie
    - name: Alice
      slug: alice
  "});

  document.move_item("", 0, 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
        slug: alice
      - name: Charlie # VIP
        slug: charlie
    "}
  );
}

#[test]
fn test_move_item_preserves_preceding_section_comment() {
  let mut document = parse(indoc! {"
    ## Section A
    - name: Alice

    ## Section B
    - name: Bob
  "});

  document.move_item("", 1, 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      ## Section B
      - name: Bob

      ## Section A
      - name: Alice
    "}
  );
}
