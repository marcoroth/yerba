mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_location_of_scalar() {
  let document = parse(indoc! {"
    name: Alice
    port: 5432
  "});

  let info = document.get_node_info("name");

  assert_eq!(info.location.start_line, 1);
  assert_eq!(info.location.start_column, 6);
  assert_eq!(info.location.end_line, 1);
  assert_eq!(info.location.end_column, 11);
}

#[test]
fn test_location_of_nested_scalar() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  let info = document.get_node_info("database.host");

  assert_eq!(info.location.start_line, 2);
  assert_eq!(info.location.start_column, 8);
  assert_eq!(info.location.end_line, 2);
  assert_eq!(info.location.end_column, 17);
}

#[test]
fn test_location_of_map() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  let info = document.get_node_info("database");

  assert_eq!(info.location.start_line, 2);
  assert_eq!(info.location.end_line, 3);
}

#[test]
fn test_location_of_sequence() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  let info = document.get_node_info("tags");

  assert_eq!(info.location.start_line, 2);
  assert_eq!(info.location.end_line, 3);
}

#[test]
fn test_location_of_indexed_item() {
  let document = parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  let info = document.get_node_info("[1].title");

  assert_eq!(info.location.start_line, 4);
  assert_eq!(info.location.start_column, 9);
  assert_eq!(info.location.end_line, 4);
  assert_eq!(info.location.end_column, 15);
}

#[test]
fn test_location_of_block_scalar() {
  let document = parse(indoc! {"
    - id: talk-1
      description: |-
        First line.
        Second line.
        Third line.
    - id: talk-2
  "});

  let info = document.get_node_info("[0].description");

  assert_eq!(info.location.start_line, 2);
  assert_eq!(info.location.end_line, 5);
}

#[test]
fn test_location_of_root_map_item() {
  let document = parse(indoc! {"
    - id: talk-1
      title: First
      description: |-
        Some text.
        More text.
    - id: talk-2
  "});

  let info = document.get_node_info("[0]");

  assert_eq!(info.location.start_line, 1);
  assert_eq!(info.location.end_line, 5);
}

#[test]
fn test_location_has_byte_offsets() {
  let document = parse(indoc! {"
    name: Alice
  "});

  let info = document.get_node_info("name");

  assert_eq!(info.location.start_offset, 6);
  assert_eq!(info.location.end_offset, 11);
}

#[test]
fn test_location_of_second_line_scalar() {
  let document = parse(indoc! {"
    name: Alice
    port: 5432
  "});

  let info = document.get_node_info("port");

  assert_eq!(info.location.start_line, 2);
  assert_eq!(info.location.start_column, 6);
  assert_eq!(info.location.start_offset, 18);
  assert_eq!(info.location.end_offset, 22);
}

#[test]
fn test_resolve_selectors_for_locations() {
  let document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
    - id: c
      title: Third
  "});

  let selectors = document.resolve_selectors("[].title");

  assert_eq!(selectors, vec!["[0].title", "[1].title", "[2].title"]);
}

#[test]
fn test_resolve_selectors_for_nested_locations() {
  let document = parse(indoc! {"
    - speakers:
        - Alice
        - Bob
    - speakers:
        - Charlie
  "});

  let selectors = document.resolve_selectors("[].speakers[]");

  assert_eq!(selectors, vec!["[0].speakers[0]", "[0].speakers[1]", "[1].speakers[0]"]);
}

#[test]
fn test_location_of_each_resolved_selector() {
  let document = parse(indoc! {"
    - id: a
    - id: b
    - id: c
  "});

  let selectors = document.resolve_selectors("[]");

  assert_eq!(selectors.len(), 3);

  let info0 = document.get_node_info(&selectors[0]);
  let info1 = document.get_node_info(&selectors[1]);
  let info2 = document.get_node_info(&selectors[2]);

  assert_eq!(info0.location.start_line, 1);
  assert_eq!(info1.location.start_line, 2);
  assert_eq!(info2.location.start_line, 3);
}
