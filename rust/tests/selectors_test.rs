mod support;
use indoc::indoc;
use support::parse;

fn get_selectors(yaml: &str) -> Vec<String> {
  parse(yaml).selectors()
}

#[test]
fn test_selectors_simple_map() {
  let selectors = get_selectors(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  assert_eq!(selectors, vec!["host", "name", "port"]);
}

#[test]
fn test_selectors_nested_map() {
  let selectors = get_selectors(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert_eq!(selectors, vec!["database", "database.host", "database.port"]);
}

#[test]
fn test_selectors_sequence_of_scalars() {
  let selectors = get_selectors(indoc! {"
    tags:
      - ruby
      - rust
  "});

  assert_eq!(selectors, vec!["tags", "tags[]"]);
}

#[test]
fn test_selectors_sequence_of_objects() {
  let selectors = get_selectors(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  assert_eq!(selectors, vec!["[]", "[].id", "[].title"]);
}

#[test]
fn test_selectors_nested_sequence() {
  let selectors = get_selectors(indoc! {"
    - id: talk-1
      speakers:
        - name: Alice
          slug: alice
  "});

  assert_eq!(
    selectors,
    vec![
      "[]",
      "[].id",
      "[].speakers",
      "[].speakers[]",
      "[].speakers[].name",
      "[].speakers[].slug",
    ]
  );
}

#[test]
fn test_selectors_mixed_structure() {
  let selectors = get_selectors(indoc! {"
    database:
      host: localhost
      port: 5432
    tags:
      - ruby
      - rust
  "});

  assert_eq!(
    selectors,
    vec!["database", "database.host", "database.port", "tags", "tags[]"]
  );
}

#[test]
fn test_selectors_deep_nesting() {
  let selectors = get_selectors(indoc! {"
    - id: talk-1
      talks:
        - title: Sub Talk
          speakers:
            - Alice
  "});

  assert_eq!(
    selectors,
    vec![
      "[]",
      "[].id",
      "[].talks",
      "[].talks[]",
      "[].talks[].speakers",
      "[].talks[].speakers[]",
      "[].talks[].title",
    ]
  );
}
