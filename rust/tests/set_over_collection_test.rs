mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_set_replaces_a_block_map() {
  let mut document = parse(indoc! {"
    venue:
      city: B
      country: DE
    name: x
  "});

  document.set("venue", "none").unwrap();

  assert_eq!(document.to_string(), "venue: none\nname: x\n");
}

#[test]
fn test_set_does_not_rename_the_first_key() {
  let mut document = parse(indoc! {"
    venue:
      city: B
    name: x
  "});

  document.set("venue", "none").unwrap();

  assert!(!document.to_string().contains("none: B"), "{}", document.to_string());
  assert_eq!(document.get("venue"), Some("none".to_string()));
}

#[test]
fn test_set_replaces_a_block_sequence() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rails
    name: x
  "});

  document.set("tags", "none").unwrap();

  assert_eq!(document.to_string(), "tags: none\nname: x\n");
}

#[test]
fn test_set_replaces_a_flow_map() {
  let mut document = parse("venue: {city: B, country: DE}\nname: x\n");

  document.set("venue", "none").unwrap();

  assert_eq!(document.to_string(), "venue: none\nname: x\n");
}

#[test]
fn test_set_replaces_a_flow_sequence() {
  let mut document = parse("tags: [ruby, rails]\nname: x\n");

  document.set("tags", "none").unwrap();

  assert_eq!(document.to_string(), "tags: none\nname: x\n");
}

#[test]
fn test_set_replaces_a_nested_collection_keeping_indentation() {
  let mut document = parse(indoc! {"
    a:
      b:
        c:
          d: 1
    e: 2
  "});

  document.set("a.b.c", "none").unwrap();

  assert_eq!(document.to_string(), "a:\n  b:\n    c: none\ne: 2\n");
}

#[test]
fn test_set_replaces_a_sequence_entry_that_is_a_map() {
  let mut document = parse(indoc! {"
    - id: 1
      name: x
    - id: 2
  "});

  document.set("[0]", "none").unwrap();

  assert_eq!(document.to_string(), "- none\n- id: 2\n");
}

#[test]
fn test_set_replaces_a_flow_sequence_entry_that_is_a_map() {
  let mut document = parse("items: [{a: 1}, {b: 2}]\n");

  document.set("items[0]", "none").unwrap();

  assert_eq!(document.to_string(), "items: [none, {b: 2}]\n");
}

#[test]
fn test_set_keeps_a_comment_on_the_key_line() {
  let mut document = parse(indoc! {"
    venue: # why
      city: B
    name: x
  "});

  document.set("venue", "none").unwrap();

  assert_eq!(document.to_string(), "venue: none # why\nname: x\n");
}

#[test]
fn test_set_keeps_a_comment_that_follows_the_collection() {
  let mut document = parse(indoc! {"
    venue:
      city: B
    # note
    name: x
  "});

  document.set("venue", "none").unwrap();

  assert_eq!(document.to_string(), "venue: none\n# note\nname: x\n");
}

#[test]
fn test_set_keeps_a_blank_line_after_the_collection() {
  let mut document = parse("venue:\n  city: B\n\nname: x\n");

  document.set("venue", "none").unwrap();

  assert_eq!(document.to_string(), "venue: none\n\nname: x\n");
}

#[test]
fn test_set_replaces_a_collection_at_the_end_without_a_trailing_newline() {
  let mut document = parse("name: x\nvenue:\n  city: B");

  document.set("venue", "none").unwrap();

  assert_eq!(document.to_string(), "name: x\nvenue: none");
}

#[test]
fn test_set_refuses_the_document_root() {
  let mut document = parse("venue:\n  city: B\n");

  assert!(document.set("", "none").is_err());
  assert_eq!(document.to_string(), "venue:\n  city: B\n");
}

#[test]
fn test_set_plain_replaces_a_collection() {
  let mut document = parse("venue:\n  city: B\nname: x\n");

  document.set_plain("venue", "5432").unwrap();

  assert_eq!(document.to_string(), "venue: 5432\nname: x\n");
}

#[test]
fn test_set_all_replaces_every_matching_collection() {
  let mut document = parse(indoc! {"
    - venue:
        city: B
    - venue:
        city: L
  "});

  document.set_all("[].venue", "none").unwrap();

  assert_eq!(document.to_string(), "- venue: none\n- venue: none\n");
}

#[test]
fn test_setting_a_scalar_is_unchanged() {
  let mut document = parse("name: x\nage: 5\n");

  document.set("name", "y").unwrap();
  document.set("age", "6").unwrap();

  assert_eq!(document.to_string(), "name: y\nage: 6\n");
}

#[test]
fn test_setting_a_scalar_keeps_its_quote_style() {
  let mut document = parse("a: \"x\"\nb: 'x'\nc: x\n");

  document.set("a", "y").unwrap();
  document.set("b", "y").unwrap();
  document.set("c", "y").unwrap();

  assert_eq!(document.to_string(), "a: \"y\"\nb: 'y'\nc: y\n");
}

#[test]
fn test_setting_a_block_scalar_is_unchanged() {
  let mut document = parse("text: |-\n  hello\n");

  document.set("text", "bye").unwrap();

  assert_eq!(document.get("text"), Some("bye".to_string()));
}
