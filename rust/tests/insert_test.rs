mod support;
use indoc::indoc;
use support::parse;
use yerba::InsertPosition;

#[test]
fn test_insert_key_at_end() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document.insert_into("database.ssl", "true", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        ssl: true
    "}
  );
}

#[test]
fn test_insert_key_at_index() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  document.insert_into("database.ssl", "true", InsertPosition::At(0)).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        ssl: true
        host: localhost
        port: 5432
    "}
  );
}

#[test]
fn test_insert_key_after() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document.insert_into("database.ssl", "true", InsertPosition::After("host".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        ssl: true
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_insert_key_before() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
      name: myapp
  "});

  document
    .insert_into("database.ssl", "true", InsertPosition::Before("port".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        ssl: true
        port: 5432
        name: myapp
    "}
  );
}

#[test]
fn test_insert_key_preserves_comments() {
  let mut document = parse(indoc! {"
    # Config
    database:
      host: localhost
      port: 5432
    # End
  "});

  document.insert_into("database.ssl", "true", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      # Config
      database:
        host: localhost
        port: 5432
        ssl: true
      # End
    "}
  );
}

#[test]
fn test_insert_duplicate_key_errors() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert!(document.insert_into("database.host", "newvalue", InsertPosition::Last).is_err());
}

#[test]
fn test_insert_root_level_key() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  document.insert_into("ssl", "true", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      port: 5432
      ssl: true
    "}
  );
}

#[test]
fn test_insert_key_after_nonexistent_errors() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
  "});

  assert!(document
    .insert_into("database.ssl", "true", InsertPosition::After("missing".to_string()),)
    .is_err());
}

#[test]
fn test_insert_from_sort_order_middle() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      name: myapp
  "});

  let order = vec!["host".to_string(), "port".to_string(), "name".to_string()];

  document.insert_into("database.port", "5432", InsertPosition::FromSortOrder(order)).unwrap();

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
fn test_insert_from_sort_order_first() {
  let mut document = parse(indoc! {"
    database:
      port: 5432
      name: myapp
  "});

  let order = vec!["host".to_string(), "port".to_string(), "name".to_string()];

  document
    .insert_into("database.host", "localhost", InsertPosition::FromSortOrder(order))
    .unwrap();

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
fn test_insert_from_sort_order_key_not_in_order() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  let order = vec!["host".to_string(), "port".to_string()];

  document.insert_into("database.ssl", "true", InsertPosition::FromSortOrder(order)).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
        ssl: true
    "}
  );
}

#[test]
fn test_insert_into_nested_map() {
  let mut document = parse(indoc! {"
    settings:
      debug: true
  "});

  document.insert_into("settings.db_host", "localhost", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      settings:
        debug: true
        db_host: localhost
    "}
  );
}

#[test]
fn test_insert_into_deeply_nested_map() {
  let mut document = parse(indoc! {"
    a:
      b:
        c: 1
  "});

  document.insert_into("a.b.d", "2", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      a:
        b:
          c: 1
          d: 2
    "}
  );
}

#[test]
fn test_insert_into_sequence_at_end() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.insert_into("tags", "yaml", InsertPosition::Last).unwrap();

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
fn test_insert_into_sequence_at_index() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.insert_into("tags", "yaml", InsertPosition::At(0)).unwrap();

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
fn test_insert_into_sequence_before() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.insert_into("tags", "yaml", InsertPosition::Before("rust".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - yaml
        - rust
    "}
  );
}

#[test]
fn test_insert_into_sequence_after() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rust
  "});

  document.insert_into("tags", "yaml", InsertPosition::After("ruby".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      tags:
        - ruby
        - yaml
        - rust
    "}
  );
}

#[test]
fn test_insert_with_bracket_index_path() {
  let mut document = parse(indoc! {"
    - id: first
      speakers:
        - Alice
    - id: second
      speakers:
        - Bob
  "});

  document.insert_into("[1].speakers", "Charlie", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        speakers:
          - Alice
      - id: second
        speakers:
          - Bob
          - Charlie
    "}
  );
}

#[test]
fn test_insert_with_bracket_index_after() {
  let mut document = parse(indoc! {"
    - id: first
      speakers:
        - Alice
        - Charlie
  "});

  document.insert_into("[0].speakers", "Bob", InsertPosition::After("Alice".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        speakers:
          - Alice
          - Bob
          - Charlie
    "}
  );
}

#[test]
fn test_insert_with_bracket_index_before() {
  let mut document = parse(indoc! {"
    - id: first
      speakers:
        - Alice
        - Charlie
  "});

  document
    .insert_into("[0].speakers", "Bob", InsertPosition::Before("Charlie".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: first
        speakers:
          - Alice
          - Bob
          - Charlie
    "}
  );
}

#[test]
fn test_insert_after_condition_on_object_sequence() {
  let mut document = parse(indoc! {"
    - name: Alice
      slug: alice
    - name: Charlie
      slug: charlie
  "});

  document
    .insert_into("", "name: Bob\n  slug: bob", InsertPosition::AfterCondition(".name == Alice".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
        slug: alice
      - name: Bob
        slug: bob
      - name: Charlie
        slug: charlie
    "}
  );
}

#[test]
fn test_insert_before_condition_on_object_sequence() {
  let mut document = parse(indoc! {"
    - name: Alice
      slug: alice
    - name: Charlie
      slug: charlie
  "});

  document
    .insert_into("", "name: Bob\n  slug: bob", InsertPosition::BeforeCondition(".name == Charlie".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
        slug: alice
      - name: Bob
        slug: bob
      - name: Charlie
        slug: charlie
    "}
  );
}

#[test]
fn test_insert_after_condition_with_bracket_path() {
  let mut document = parse(indoc! {"
    - id: talk-1
      speakers:
        - name: Alice
        - name: Charlie
  "});

  document
    .insert_into("[0].speakers", "name: Bob", InsertPosition::AfterCondition(".name == Alice".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        speakers:
          - name: Alice
          - name: Bob
          - name: Charlie
    "}
  );
}

#[test]
fn test_insert_condition_no_match_errors() {
  let mut document = parse(indoc! {"
    - name: Alice
    - name: Bob
  "});

  let result = document.insert_into("", "name: Charlie", InsertPosition::AfterCondition(".name == Missing".to_string()));

  assert!(result.is_err());
}

#[test]
fn test_insert_multiline_map_into_sequence() {
  let mut document = parse(indoc! {"
    - name: Alice
      slug: alice
  "});

  document.insert_into("", "name: Bob\nslug: bob\ngithub: bob", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
        slug: alice
      - name: Bob
        slug: bob
        github: bob
    "}
  );
}

#[test]
fn test_insert_multiline_map_with_nested_array() {
  let mut document = parse(indoc! {"
    - id: talk-1
      title: First
  "});

  document
    .insert_into("", "id: talk-2\ntitle: Second\nspeakers:\n  - Alice\n  - Bob", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        title: First
      - id: talk-2
        title: Second
        speakers:
          - Alice
          - Bob
    "}
  );
}

#[test]
fn test_insert_multiline_at_position() {
  let mut document = parse(indoc! {"
    - name: Alice
      slug: alice
    - name: Charlie
      slug: charlie
  "});

  document.insert_into("", "name: Bob\nslug: bob", InsertPosition::At(1)).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
        slug: alice
      - name: Bob
        slug: bob
      - name: Charlie
        slug: charlie
    "}
  );
}

#[test]
fn test_insert_multiline_after_condition() {
  let mut document = parse(indoc! {"
    - name: Alice
      slug: alice
    - name: Charlie
      slug: charlie
  "});

  document
    .insert_into("", "name: Bob\nslug: bob", InsertPosition::AfterCondition(".name == Alice".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice
        slug: alice
      - name: Bob
        slug: bob
      - name: Charlie
        slug: charlie
    "}
  );
}

#[test]
fn test_insert_multiline_into_nested_sequence() {
  let mut document = parse(indoc! {"
    talks:
      - id: talk-1
        title: First
  "});

  document
    .insert_into("talks", "id: talk-2\ntitle: Second\nspeakers:\n  - Alice", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      talks:
        - id: talk-1
          title: First
        - id: talk-2
          title: Second
          speakers:
            - Alice
    "}
  );
}

#[test]
fn test_insert_after_relative_condition() {
  let mut document = parse(indoc! {"
    - id: talk-1
      kind: keynote
    - id: talk-2
      kind: talk
    - id: talk-3
      kind: keynote
  "});

  document
    .insert_into("", "id: talk-4\nkind: talk", InsertPosition::AfterCondition(".id == talk-1".to_string()))
    .unwrap();

  let ids: Vec<String> = document.get_all("[].id");
  assert_eq!(ids, vec!["talk-1", "talk-4", "talk-2", "talk-3"]);
}

#[test]
fn test_insert_before_relative_condition() {
  let mut document = parse(indoc! {"
    - id: talk-1
    - id: talk-2
    - id: talk-3
  "});

  document
    .insert_into("", "id: talk-new", InsertPosition::BeforeCondition(".id == talk-3".to_string()))
    .unwrap();

  let ids: Vec<String> = document.get_all("[].id");
  assert_eq!(ids, vec!["talk-1", "talk-2", "talk-new", "talk-3"]);
}

#[test]
fn test_insert_condition_requires_dot_prefix() {
  let mut document = parse(indoc! {"
    - id: talk-1
    - id: talk-2
  "});

  let result = document.insert_into("", "id: talk-3", InsertPosition::AfterCondition("id == talk-1".to_string()));

  assert!(result.is_err());
}

#[test]
fn test_insert_with_wildcard_adds_key_to_all_entries() {
  let mut document = parse(indoc! {"
    - title: Talk 1
      date: 2024-01-01
    - title: Talk 2
      date: 2024-01-02
  "});

  document.insert_into("[].language", "pt", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - title: Talk 1
        date: 2024-01-01
        language: pt
      - title: Talk 2
        date: 2024-01-02
        language: pt
    "}
  );
}

#[test]
fn test_insert_with_wildcard_after_position() {
  let mut document = parse(indoc! {"
    - title: Talk 1
      date: 2024-01-01
      speakers:
        - Alice
    - title: Talk 2
      date: 2024-01-02
      speakers:
        - Bob
  "});

  document.insert_into("[].language", "pt", InsertPosition::After("date".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - title: Talk 1
        date: 2024-01-01
        language: pt
        speakers:
          - Alice
      - title: Talk 2
        date: 2024-01-02
        language: pt
        speakers:
          - Bob
    "}
  );
}

#[test]
fn test_insert_with_wildcard_skips_entries_that_already_have_key() {
  let mut document = parse(indoc! {"
    - title: Talk 1
      language: en
    - title: Talk 2
  "});

  document.insert_into("[].language", "pt", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - title: Talk 1
        language: en
      - title: Talk 2
        language: pt
    "}
  );
}

#[test]
fn test_insert_with_nested_wildcard() {
  let mut document = parse(indoc! {"
    - title: Session 1
      talks:
        - title: Talk A
          date: 2024-01-01
        - title: Talk B
          date: 2024-01-02
    - title: Session 2
      speakers:
        - Charlie
  "});

  document
    .insert_into("[].talks[].language", "pt", InsertPosition::After("date".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - title: Session 1
        talks:
          - title: Talk A
            date: 2024-01-01
            language: pt
          - title: Talk B
            date: 2024-01-02
            language: pt
      - title: Session 2
        speakers:
          - Charlie
    "}
  );
}

#[test]
fn test_insert_with_wildcard_no_matches_is_ok() {
  let mut document = parse(indoc! {"
    - title: Talk 1
    - title: Talk 2
  "});

  let result = document.insert_into("[].talks[].language", "pt", InsertPosition::Last);

  assert!(result.is_ok());
}

#[test]
fn test_insert_block_value_in_nested_map() {
  let mut document = parse(indoc! {"
    - id: talk-1
      title: First Talk
  "});

  document.insert_into("[0].speakers", "- Alice\n- Bob", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        title: First Talk
        speakers:
          - Alice
          - Bob
    "}
  );
}

#[test]
fn test_insert_block_value_in_deeply_nested_map() {
  let mut document = parse(indoc! {"
    app:
      events:
        - id: event-1
          title: My Event
  "});

  document.insert_into("app.events[0].speakers", "- Alice", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      app:
        events:
          - id: event-1
            title: My Event
            speakers:
              - Alice
    "}
  );
}

#[test]
fn test_insert_after_preserves_inline_comment() {
  let mut document = parse(indoc! {r#"
    - id: "vinash"
      title: "Lightning Talk: Vinash" # TODO: missing last name
      event_name: "Garden City Ruby 2014"
  "#});

  document
    .insert_into("[0].kind", "lightning_talk", InsertPosition::After("title".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: "vinash"
        title: "Lightning Talk: Vinash" # TODO: missing last name
        kind: lightning_talk
        event_name: "Garden City Ruby 2014"
    "#}
  );
}

#[test]
fn test_insert_last_preserves_inline_comment_on_last_entry() {
  let mut document = parse(indoc! {r#"
    name: "Alice"
    age: 30 # years old
  "#});

  document.insert_into("email", "alice@example.com", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      name: "Alice"
      age: 30 # years old
      email: alice@example.com
    "#}
  );
}

#[test]
fn test_insert_after_without_comment_still_works() {
  let mut document = parse(indoc! {r#"
    - id: "talk-1"
      title: "First Talk"
      event_name: "Conf 2024"
  "#});

  document.insert_into("[0].kind", "talk", InsertPosition::After("title".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: "talk-1"
        title: "First Talk"
        kind: talk
        event_name: "Conf 2024"
    "#}
  );
}

#[test]
fn test_insert_preserves_comment_on_first_entry() {
  let mut document = parse(indoc! {r#"
    name: "Alice" # primary name
    age: 30
  "#});

  document
    .insert_into("email", "alice@example.com", InsertPosition::After("name".to_string()))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      name: "Alice" # primary name
      email: alice@example.com
      age: 30
    "#}
  );
}

#[test]
fn test_insert_preserves_comment_on_middle_entry() {
  let mut document = parse(indoc! {r#"
    name: "Alice"
    age: 30 # years old
    email: "alice@example.com"
  "#});

  document.insert_into("role", "admin", InsertPosition::After("age".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      name: "Alice"
      age: 30 # years old
      role: admin
      email: "alice@example.com"
    "#}
  );
}

#[test]
fn test_insert_preserves_multiple_comments_on_different_entries() {
  let mut document = parse(indoc! {r#"
    name: "Alice" # first name only
    age: 30 # years old
  "#});

  document.insert_into("email", "alice@example.com", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      name: "Alice" # first name only
      age: 30 # years old
      email: alice@example.com
    "#}
  );
}

#[test]
fn test_insert_before_preserves_inline_comment() {
  let mut document = parse(indoc! {r#"
    name: "Alice"
    age: 30 # years old
    email: "alice@example.com"
  "#});

  document.insert_into("role", "admin", InsertPosition::Before("age".to_string())).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      name: "Alice"
      role: admin
      age: 30 # years old
      email: "alice@example.com"
    "#}
  );
}

#[test]
fn test_insert_into_sequence_preserves_inline_comment() {
  let mut document = parse(indoc! {"
    - name: Alice # admin
    - name: Bob
  "});

  document.insert_into("", "name: Carol", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - name: Alice # admin
      - name: Bob
      - name: Carol
    "}
  );
}

#[test]
fn test_insert_at_index_preserves_inline_comment() {
  let mut document = parse(indoc! {r#"
    name: "Alice"
    age: 30 # years old
    email: "alice@example.com"
  "#});

  document.insert_into("role", "admin", InsertPosition::At(1)).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      name: "Alice"
      role: admin
      age: 30 # years old
      email: "alice@example.com"
    "#}
  );
}

#[test]
fn test_a_one_key_mapping_fragment_is_inserted_as_a_mapping() {
  let mut document = parse("name: Alice\n");

  document.insert_fragment_into("venue", "country: DE", InsertPosition::Last).unwrap();

  assert_eq!(document.to_string(), "name: Alice\nvenue:\n  country: DE\n");
  assert_eq!(document.get_value("venue.country"), Some(yaml_serde::Value::String("DE".to_string())));
}

#[test]
fn test_the_same_text_stays_a_string_when_it_is_a_value() {
  let mut document = parse("name: Alice\n");

  document.insert_into("venue", "country: DE", InsertPosition::Last).unwrap();

  assert_eq!(document.to_string(), "name: Alice\nvenue: \"country: DE\"\n");
  assert_eq!(document.get_value("venue"), Some(yaml_serde::Value::String("country: DE".to_string())));
}

#[test]
fn test_a_flow_collection_fragment_stays_inline() {
  let mut document = parse("name: Alice\n");

  document.insert_fragment_into("venue", "{country: DE}", InsertPosition::Last).unwrap();

  assert_eq!(document.to_string(), "name: Alice\nvenue: {country: DE}\n");
}
