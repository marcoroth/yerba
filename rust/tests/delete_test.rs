mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_delete_key() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      name: myapp
    "}
  );
}

#[test]
fn test_delete_nested_key() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document.delete("database.port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        name: myapp
    "}
  );
}

#[test]
fn test_delete_preserves_comments() {
  let mut document = parse(indoc! {"
    # Config
    host: localhost
    port: 5432
    # End
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Config
      host: localhost
      # End
    "}
  );
}

#[test]
fn test_delete_nonexistent_key() {
  let mut document = parse("host: localhost\n");

  assert!(document.delete("missing").is_err());
}

#[test]
fn test_delete_orphans_trailing_comment() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    # port docs
    name: myapp
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      # port docs
      name: myapp
    "}
  );
}

#[test]
fn test_delete_preserves_inline_comment_on_own_line() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432 # important
    name: myapp
  "});

  document.delete("port").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      # important
      name: myapp
    "}
  );
}

#[test]
fn test_delete_with_bracket_index_path() {
  let mut document = parse(indoc! {"
    - id: first
      title: A
      description: remove me
    - id: second
      title: B
  "});

  document.delete("[0].description").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        title: A
      - id: second
        title: B
    "}
  );
}

#[test]
fn test_delete_root_sequence_item_by_index() {
  let mut document = parse(indoc! {"
    - name: Entry 1
    - name: Entry 2
      description: remove me too
  "});

  document.delete("[1]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Entry 1
    "}
  );
}

#[test]
fn test_delete_nested_sequence_item_by_index() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - go
  "});

  document.delete("tags[1]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - go
    "}
  );
}

#[test]
fn test_delete_nested_sequence_item_by_index_from_end() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - go
  "});

  let error = document.delete("tags[-4]").unwrap_err();

  assert_eq!(error.to_string(), "selector not found: tags[-4]");

  document.delete("tags[-1]").unwrap();

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
fn test_delete_only_sequence_item_replaces_with_empty_sequence() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
  "});

  document.delete("tags[0]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
       tags: []
    "}
  );
}

#[test]
fn test_delete_only_top_level_sequence_item_replaces_with_empty_sequence() {
  let mut document = parse(indoc! {"
      # TODO: Add videos and recordings
      ---
      - video_id: delete-me
        title: About to be Deleted
  "});

  document.delete("[0]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
       # TODO: Add videos and recordings
       ---
       []
    "}
  );
}

#[test]
fn test_delete_one_of_two_sequence_items_does_not_collapse() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.delete("tags[0]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
       tags:
         - rust
    "}
  );
}

#[test]
fn test_delete_only_sequence_item_preserves_key_comment() {
  let mut document = parse(indoc! {"
    tags: # note
      - ruby
  "});

  document.delete("tags[0]").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
       tags: [] # note
    "}
  );
}

#[test]
fn test_delete_one_of_two_map_keys_does_not_collapse() {
  let mut document = parse(indoc! {"
    speaker:
      name: Alice
      role: host
  "});

  document.delete("speaker.name").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
       speaker:
         role: host
    "}
  );
}

#[test]
fn test_delete_only_map_key_replaces_with_empty_map() {
  let mut document = parse(indoc! {"
    speaker:
      name: Alice
  "});

  document.delete("speaker.name").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
       speaker: {}
    "}
  );
}

#[test]
fn test_delete_wildcard_removes_key_from_all_entries() {
  let mut document = parse(indoc! {r#"
    - name: "Alice"
      github: "aalice"
      slug: "alice"
    - name: "Bob"
      github: "bob123"
      slug: "bob"
  "#});

  document.delete("[].github").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - name: "Alice"
        slug: "alice"
      - name: "Bob"
        slug: "bob"
    "#}
  );
}

#[test]
fn test_delete_wildcard_skips_entries_without_key() {
  let mut document = parse(indoc! {r#"
    - name: "Alice"
      github: "aalice"
    - name: "Bob"
    - name: "Charlie"
      github: "charlie"
  "#});

  document.delete("[].github").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - name: "Alice"
      - name: "Bob"
      - name: "Charlie"
    "#}
  );
}

#[test]
fn test_delete_wildcard_nested_path() {
  let mut document = parse(indoc! {r#"
    items:
      - name: "A"
        extra: "remove"
      - name: "B"
        extra: "also remove"
  "#});

  document.delete("items[].extra").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      items:
        - name: "A"
        - name: "B"
    "#}
  );
}

#[test]
fn test_delete_wildcard_no_matches_is_ok() {
  let mut document = parse(indoc! {r#"
    - name: "Alice"
    - name: "Bob"
  "#});

  let result = document.delete("[].nonexistent");

  assert!(result.is_ok());
  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - name: "Alice"
      - name: "Bob"
    "#}
  );
}

#[test]
fn test_delete_wildcard_single_parent_no_match_is_ok() {
  let mut document = parse(indoc! {r#"
    - tiers:
        - name: "Gold"
          sponsors:
            - name: "Sponsor A"
  "#});

  let result = document.delete("[].tiers[].sponsors[].description");

  assert!(result.is_ok());
  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - tiers:
          - name: "Gold"
            sponsors:
              - name: "Sponsor A"
    "#}
  );
}

#[test]
fn test_delete_non_wildcard_missing_key_errors() {
  let mut document = parse(indoc! {"
    name: Alice
  "});

  let result = document.delete("nonexistent");

  assert!(result.is_err());
}

#[test]
fn test_delete_key_with_sequence_value() {
  let mut document = parse(indoc! {"
    name: Helvetic Ruby
    kind: conference
    youtube_channels:
    - id: UChetoakh7nU0EXrKN8QFUUA
      name: HelveticRuby
    - id: UC123
      name: Other
  "});

  document.delete("youtube_channels").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: Helvetic Ruby
      kind: conference
    "}
  );
}

#[test]
fn test_delete_key_with_empty_sequence_value() {
  let mut document = parse(indoc! {"
    name: Helvetic Ruby
    youtube_channels:
  "});

  document.delete("youtube_channels").unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      name: Helvetic Ruby
    "}
  );
}
