mod support;
use indoc::indoc;
use support::parse;
use yerba::BlankLineOptions;

fn before(keys: &[&str]) -> BlankLineOptions {
  BlankLineOptions {
    before: keys.iter().map(|key| key.to_string()).collect(),
    ..Default::default()
  }
}

#[test]
fn test_add_blank_lines_between_entries() {
  let mut document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
  "});

  document.enforce_blank_lines("", 1).unwrap();

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
fn test_remove_blank_lines_between_entries() {
  let mut document = parse(indoc! {"
    - id: a
      title: First

    - id: b
      title: Second
  "});

  document.enforce_blank_lines("", 0).unwrap();

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
fn test_blank_lines_noop_when_already_correct() {
  let mut document = parse(indoc! {"
    - id: a

    - id: b
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a

      - id: b
    "}
  );
}

#[test]
fn test_blank_lines_nested_sequence() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  document.enforce_blank_lines("tags", 1).unwrap();

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
fn test_blank_lines_single_entry_noop() {
  let mut document = parse(indoc! {"
    - id: only
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(document.to_string(), "- id: only\n");
}

#[test]
fn test_blank_lines_multiple_blank_lines() {
  let mut document = parse(indoc! {"
    - id: a
    - id: b
    - id: c
  "});

  document.enforce_blank_lines("", 2).unwrap();

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
fn test_blank_lines_preserves_comments() {
  let mut document = parse(indoc! {"
    # comment
    - id: a
    - id: b
    # end
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # comment
      - id: a

      - id: b
      # end
    "}
  );
}

#[test]
fn test_blank_lines_with_multi_line_entries() {
  let mut document = parse(indoc! {"
    - id: a
      title: First
      speakers:
        - Alice
    - id: b
      title: Second
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        title: First
        speakers:
          - Alice

      - id: b
        title: Second
    "}
  );
}

#[test]
fn test_blank_lines_normalizes_extra_blanks() {
  let mut document = parse(indoc! {"
    - id: a



    - id: b
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a

      - id: b
    "}
  );
}

#[test]
fn test_blank_lines_top_level_only() {
  let mut document = parse(indoc! {"
    - id: a
      speakers:
        - Alice
        - Bob
    - id: b
      speakers:
        - Charlie
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        speakers:
          - Alice
          - Bob

      - id: b
        speakers:
          - Charlie
    "}
  );
}

#[test]
fn test_blank_lines_nested_only() {
  let mut document = parse(indoc! {"
    - id: a
      speakers:
        - Alice
        - Bob
    - id: b
      speakers:
        - Charlie
        - Diana
  "});

  document.enforce_blank_lines("[].speakers", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
        speakers:
          - Alice

          - Bob
      - id: b
        speakers:
          - Charlie

          - Diana
    "}
  );
}

#[test]
fn test_blank_lines_remove_from_map() {
  let mut document = parse(indoc! {"
    host: localhost

    port: 5432

    name: myapp
  "});

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      port: 5432
      name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_add_to_map() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  document.enforce_blank_lines("", 1).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost

      port: 5432

      name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_map_noop_when_correct() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  let original = document.to_string();

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_blank_lines_nested_map() {
  let mut document = parse(indoc! {"
    database:
      host: localhost

      port: 5432

      name: myapp
  "});

  document.enforce_blank_lines("database", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_map_with_comments() {
  let mut document = parse(indoc! {"
    # Database config
    host: localhost

    port: 5432

    # App name
    name: myapp
  "});

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Database config
      host: localhost
      port: 5432

      # App name
      name: myapp
    "}
  );
}

#[test]
fn test_blank_lines_map_normalizes_multiple() {
  let mut document = parse(indoc! {"
    host: localhost



    port: 5432
  "});

  document.enforce_blank_lines("", 0).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      port: 5432
    "}
  );
}

#[test]
fn test_blank_lines_before_only_spaces_listed_keys() {
  let mut document = parse(indoc! {"
    name: link_to
    gem: actionview
    arguments:
      - name: name
    options:
      - name: method
  "});

  document.enforce_blank_lines_with("", 1, &before(&["arguments", "options"])).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: link_to
      gem: actionview

      arguments:
        - name: name

      options:
        - name: method
    "}
  );
}

#[test]
fn test_blank_lines_before_removes_blank_lines_from_unlisted_keys() {
  let mut document = parse(indoc! {"
    name: link_to

    gem: actionview

    arguments:
      - name: name
  "});

  document.enforce_blank_lines_with("", 0, &before(&["gem"])).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: link_to
      gem: actionview

      arguments:
        - name: name
    "}
  );
}

#[test]
fn test_blank_lines_before_ignores_the_first_entry() {
  let mut document = parse(indoc! {"
    name: link_to
    gem: actionview
  "});

  document.enforce_blank_lines_with("", 1, &before(&["name"])).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: link_to
      gem: actionview
    "}
  );
}

#[test]
fn test_blank_lines_before_never_matches_sequence_entries() {
  let mut document = parse(indoc! {"
    - id: a
    - id: b
  "});

  document.enforce_blank_lines_with("", 1, &before(&["id"])).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: a
      - id: b
    "}
  );
}

#[test]
fn test_blank_lines_before_applies_to_every_wildcard_match() {
  let mut document = parse(indoc! {"
    - name: first
      options:
        - a
    - name: second
      options:
        - b
  "});

  document.enforce_blank_lines_with("[]", 1, &before(&["options"])).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: first

        options:
          - a
      - name: second

        options:
          - b
    "}
  );
}

#[test]
fn test_blank_lines_without_filter_still_spaces_every_entry() {
  let mut document = parse(indoc! {"
    name: link_to
    gem: actionview
    arguments:
      - name: name
  "});

  document.enforce_blank_lines_with("", 1, &before(&[])).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: link_to

      gem: actionview

      arguments:
        - name: name
    "}
  );
}

#[test]
fn test_blank_lines_skip_empty_leaves_placeholder_entries_packed() {
  let mut document = parse(indoc! {"
    name: content_security_policy?
    description: Returns whether a policy is present.
    tag: null
    arguments: []
    options: []
    special_behaviors: []
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        before: ["description", "tag", "arguments", "options", "special_behaviors"]
          .iter()
          .map(|key| key.to_string())
          .collect(),
        skip_empty: true,
        ..Default::default()
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: content_security_policy?

      description: Returns whether a policy is present.
      tag: null
      arguments: []
      options: []
      special_behaviors: []
    "}
  );
}

#[test]
fn test_blank_lines_skip_empty_still_spaces_populated_entries() {
  let mut document = parse(indoc! {"
    name: image_tag
    tag: null
    arguments:
      - name: source
    options: []
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        before: ["tag", "arguments", "options"].iter().map(|key| key.to_string()).collect(),
        skip_empty: true,
        ..Default::default()
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: image_tag
      tag: null

      arguments:
        - name: source
      options: []
    "}
  );
}

#[test]
fn test_blank_lines_skip_empty_treats_empty_string_as_empty() {
  let mut document = parse(indoc! {"
    name: helper
    documentation_url: \"\"
    signature: helper()
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        skip_empty: true,
        ..Default::default()
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: helper
      documentation_url: \"\"

      signature: helper()
    "}
  );
}

#[test]
fn test_blank_lines_without_skip_empty_spaces_placeholders_too() {
  let mut document = parse(indoc! {"
    name: helper
    tag: null
    arguments: []
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        before: ["tag", "arguments"].iter().map(|key| key.to_string()).collect(),
        skip_empty: false,
        ..Default::default()
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: helper

      tag: null

      arguments: []
    "}
  );
}

#[test]
fn test_blank_lines_after_separates_a_block_from_the_empty_entries_that_follow() {
  let mut document = parse(indoc! {"
    name: tag
    tag:
      name: meta
      is_void: true
    arguments: []
    options: []
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        before: ["tag", "arguments", "options"].iter().map(|key| key.to_string()).collect(),
        after: vec!["tag".to_string()],
        skip_empty: true,
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: tag

      tag:
        name: meta
        is_void: true

      arguments: []
      options: []
    "}
  );
}

#[test]
fn test_blank_lines_after_is_suppressed_when_the_trigger_is_empty() {
  let mut document = parse(indoc! {"
    name: helper
    tag: null
    arguments: []
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        after: vec!["tag".to_string()],
        skip_empty: true,
        ..Default::default()
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: helper
      tag: null
      arguments: []
    "}
  );
}

#[test]
fn test_blank_lines_after_without_skip_empty_always_separates() {
  let mut document = parse(indoc! {"
    name: helper
    tag: null
    arguments: []
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        after: vec!["tag".to_string()],
        ..Default::default()
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: helper
      tag: null

      arguments: []
    "}
  );
}

#[test]
fn test_blank_lines_before_and_after_the_same_key_sets_it_apart_on_both_sides() {
  let mut document = parse(indoc! {"
    name: helper
    tag:
      name: meta
    arguments: []
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        before: vec!["tag".to_string()],
        after: vec!["tag".to_string()],
        skip_empty: true,
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: helper

      tag:
        name: meta

      arguments: []
    "}
  );
}

#[test]
fn test_blank_lines_before_and_after_matching_the_same_gap_yields_one_blank_line() {
  let mut document = parse(indoc! {"
    tag:
      name: meta
    arguments:
      - name: source
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        before: vec!["arguments".to_string()],
        after: vec!["tag".to_string()],
        skip_empty: true,
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tag:
        name: meta

      arguments:
        - name: source
    "}
  );
}

#[test]
fn test_blank_lines_overlapping_filters_normalize_existing_extra_blank_lines() {
  let mut document = parse(indoc! {"
    tag:
      name: meta



    arguments:
      - name: source
  "});

  document
    .enforce_blank_lines_with(
      "",
      1,
      &BlankLineOptions {
        before: vec!["arguments".to_string()],
        after: vec!["tag".to_string()],
        skip_empty: true,
      },
    )
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tag:
        name: meta

      arguments:
        - name: source
    "}
  );
}
