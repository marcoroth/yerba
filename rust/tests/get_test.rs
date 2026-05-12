mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_get_plain_scalar() {
  let document = parse("host: localhost");

  assert_eq!(document.get("host"), Some("localhost".to_string()));
}

#[test]
fn test_get_double_quoted_scalar() {
  let document = parse(r#"name: "myapp""#);

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_single_quoted_scalar() {
  let document = parse("name: 'myapp'");

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_nested_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  assert_eq!(document.get("database.host"), Some("localhost".to_string()));
  assert_eq!(document.get("database.port"), Some("5432".to_string()));
}

#[test]
fn test_get_deeply_nested() {
  let document = parse(indoc! {"
    a:
      b:
        c: deep
  "});

  assert_eq!(document.get("a.b.c"), Some("deep".to_string()));
}

#[test]
fn test_get_nonexistent_path() {
  let document = parse("host: localhost");

  assert_eq!(document.get("missing"), None);
}

#[test]
fn test_get_with_brackets_returns_first() {
  let document = parse(indoc! {"
    - id: a
    - id: b
  "});

  assert_eq!(document.get("[].id"), Some("a".to_string()));
}

#[test]
fn test_get_all_with_bracket_path() {
  let document = parse(indoc! {"
    - id: a
      name: first
    - id: b
      name: second
  "});

  assert_eq!(document.get_all("[].name"), vec!["first", "second"]);
  assert_eq!(document.get_all("[].id"), vec!["a", "b"]);
}

#[test]
fn test_get_all_nested_brackets() {
  let document = parse(indoc! {"
    - speakers:
        - name: Alice
        - name: Bob
    - speakers:
        - name: Charlie
  "});

  assert_eq!(document.get_all("[].speakers[].name"), vec!["Alice", "Bob", "Charlie"]);
}

#[test]
fn test_get_all_with_key_prefix() {
  let document = parse(indoc! {"
    data:
      items:
        - name: first
        - name: second
  "});

  assert_eq!(document.get_all("data.items.[].name"), vec!["first", "second"]);
}

#[test]
fn test_get_sequence_values() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  assert_eq!(document.get_sequence_values("tags"), vec!["ruby", "rust", "yaml"]);
}

#[test]
fn test_get_sequence_values_empty() {
  let document = parse("host: localhost\n");

  assert!(document.get_sequence_values("host").is_empty());
  assert!(document.get_sequence_values("missing").is_empty());
}

#[test]
fn test_exists_returns_true_for_existing_path() {
  let document = parse("host: localhost\n");

  assert!(document.exists("host"));
}

#[test]
fn test_exists_returns_false_for_missing_path() {
  let document = parse("host: localhost\n");

  assert!(!document.exists("missing"));
}

#[test]
fn test_exists_nested_path() {
  let document = parse(indoc! {"
    database:
      host: localhost
  "});

  assert!(document.exists("database.host"));
  assert!(!document.exists("database.missing"));
}

#[test]
fn test_exists_with_brackets() {
  let document = parse(indoc! {"
    - id: a
      name: first
    - id: b
      name: second
  "});

  assert!(document.exists("[].name"));
  assert!(!document.exists("[].missing"));
}

#[test]
fn test_roundtrip_no_changes() {
  let yaml = indoc! {"
    # comment
    key: value
    nested:
      a: 1
      b: 'two'
  "};
  let document = parse(yaml);

  assert_eq!(document.to_string(), yaml);
}

#[test]
fn test_get_with_bracket_index() {
  let document = parse(indoc! {"
    - id: first
      title: A
    - id: second
      title: B
  "});

  assert_eq!(document.get("[0].title"), Some("A".to_string()));
  assert_eq!(document.get("[1].title"), Some("B".to_string()));
  assert_eq!(document.get("[2].title"), None);
}

#[test]
fn test_get_nested_bracket_path() {
  let document = parse(indoc! {"
    - id: talk-1
      speakers:
        - Alice
        - Bob
    - id: talk-2
      speakers:
        - Charlie
  "});

  assert_eq!(document.get_all("[].speakers[]"), vec!["Alice", "Bob", "Charlie"]);
  assert_eq!(document.get_all("[0].speakers[]"), vec!["Alice", "Bob"]);
  assert_eq!(document.get_all("[1].speakers[]"), vec!["Charlie"]);
}

#[test]
fn test_find_items_filters_by_field() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: keynote
      title: Opening
    - id: talk-2
      kind: talk
      title: Deep Dive
    - id: talk-3
      kind: keynote
      title: Closing
  "});

  let matches = document.filter("[]", ".kind == keynote");
  assert_eq!(matches.len(), 2);
  assert!(serde_yaml::to_string(&matches[0]).unwrap().contains("talk-1"));
  assert!(serde_yaml::to_string(&matches[0]).unwrap().contains("talk-1"));
}

#[test]
fn test_find_items_condition_on_nested_value() {
  let document = parse(indoc! {"
    - id: talk-1
      video_provider: youtube
      video_id: abc123
    - id: talk-2
      video_provider: vimeo
      video_id: def456
    - id: talk-3
      video_provider: youtube
      video_id: ghi789
  "});

  let matches = document.filter("[]", ".video_provider == youtube");
  assert_eq!(matches.len(), 2);
}

#[test]
fn test_find_items_contains_condition() {
  let document = parse(indoc! {"
    - id: talk-1
      title: Ruby on Rails
    - id: talk-2
      title: Python Basics
    - id: talk-3
      title: Advanced Ruby
  "});

  let matches = document.filter("[]", ".title contains Ruby");
  assert_eq!(matches.len(), 2);
}

#[test]
fn test_find_items_not_contains_condition() {
  let document = parse(indoc! {"
    - id: talk-1
      title: Ruby on Rails
    - id: talk-2
      title: Python Basics
  "});

  let matches = document.filter("[]", ".title not_contains Ruby");
  assert_eq!(matches.len(), 1);
  assert!(serde_yaml::to_string(&matches[0]).unwrap().contains("Python"));
}

#[test]
fn test_find_items_inequality_condition() {
  let document = parse(indoc! {"
    - id: talk-1
      status: draft
    - id: talk-2
      status: published
  "});

  let matches = document.filter("[]", ".status != draft");
  assert_eq!(matches.len(), 1);
  assert!(serde_yaml::to_string(&matches[0]).unwrap().contains("published"));
}

#[test]
fn test_find_all_returns_all_items() {
  let document = parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  let matches = document.get_values("[]");
  assert_eq!(matches.len(), 2);
}

#[test]
fn test_find_items_no_matches_returns_empty() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: talk
    - id: talk-2
      kind: talk
  "});

  let matches = document.filter("[]", ".kind == keynote");
  assert_eq!(matches.len(), 0);
}

#[test]
fn test_get_all_bracket_index_returns_scalar_for_field() {
  let document = parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  assert_eq!(document.get_all("[].title"), vec!["First", "Second"]);
  assert_eq!(document.get_all("[0].title"), vec!["First"]);
  assert_eq!(document.get_all("[1].title"), vec!["Second"]);
}

#[test]
fn test_get_all_bracket_on_scalar_array() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  assert_eq!(document.get_all("tags.[]"), vec!["ruby", "rust", "yaml"]);
}

#[test]
fn test_find_items_without_dot_prefix_does_not_match() {
  let document = parse(indoc! {"
    - id: talk-1
      kind: keynote
    - id: talk-2
      kind: talk
  "});

  let matches = document.filter("[]", "kind == keynote");
  assert_eq!(matches.len(), 0);
}

#[test]
fn test_find_items_speakers_contains_array_member() {
  let document = parse(indoc! {"
    - id: talk-1
      speakers:
        - Alice
        - Bob
    - id: talk-2
      speakers:
        - Charlie
  "});

  let matches = document.filter("[]", ".speakers contains Alice");
  assert_eq!(matches.len(), 1);
  assert!(serde_yaml::to_string(&matches[0]).unwrap().contains("talk-1"));
}

#[test]
fn test_get_sequence_key_returns_first_scalar() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - go
  "});

  assert_eq!(document.get("tags"), Some("ruby".to_string()));
}

#[test]
fn test_get_all_sequence_key_returns_all_items() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - go
  "});

  assert_eq!(document.get_all("tags.[]"), vec!["ruby", "rust", "go"]);
  assert_eq!(document.get_all("tags[]"), vec!["ruby", "rust", "go"]);
}

#[test]
fn test_get_sequence_values_returns_all_items() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - go
  "});

  assert_eq!(document.get_sequence_values("tags"), vec!["ruby", "rust", "go"]);
}

#[test]
fn test_get_nested_sequence_key_vs_brackets() {
  let document = parse(indoc! {"
    - id: talk-1
      speakers:
        - Alice
        - Bob
    - id: talk-2
      speakers:
        - Charlie
  "});

  assert_eq!(document.get_all("[].speakers"), vec!["Alice", "Charlie"]);
  assert_eq!(document.get_all("[].speakers[]"), vec!["Alice", "Bob", "Charlie"]);
}

#[test]
fn test_get_single_index_vs_all() {
  let document = parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
    - id: talk-3
      title: Third
  "});

  assert_eq!(document.get_all("[].title"), vec!["First", "Second", "Third"]);
  assert_eq!(document.get_all("[0].title"), vec!["First"]);
  assert_eq!(document.get_all("[2].title"), vec!["Third"]);
}

#[test]
fn test_get_first_item_from_nested_arrays() {
  let document = parse(indoc! {"
    - id: talk-1
      tags:
        - ruby
        - rails
    - id: talk-2
      tags:
        - python
        - django
    - id: talk-3
      tags:
        - rust
        - wasm
  "});

  assert_eq!(document.get_all("[].tags[0]"), vec!["ruby", "python", "rust"]);
  assert_eq!(document.get_all("[].tags[1]"), vec!["rails", "django", "wasm"]);

  assert_eq!(document.get_all("[].tags[]"), vec!["ruby", "rails", "python", "django", "rust", "wasm"]);
}

#[test]
fn test_get_value_returns_string_for_scalar() {
  let document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  let value = document.get_value("host").unwrap();
  assert_eq!(value, serde_yaml::Value::String("localhost".to_string()));
}

#[test]
fn test_get_value_returns_number_for_integer() {
  let document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  let value = document.get_value("port").unwrap();
  assert_eq!(value, serde_yaml::Value::Number(serde_yaml::Number::from(5432)));
}

#[test]
fn test_get_value_returns_mapping_for_object() {
  let document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
  "});

  let value = document.get_value("database").unwrap();

  if let serde_yaml::Value::Mapping(map) = value {
    assert_eq!(
      map.get(&serde_yaml::Value::String("host".to_string())),
      Some(&serde_yaml::Value::String("localhost".to_string()))
    );
    assert_eq!(
      map.get(&serde_yaml::Value::String("port".to_string())),
      Some(&serde_yaml::Value::Number(serde_yaml::Number::from(5432)))
    );
  } else {
    panic!("Expected Mapping, got: {:?}", value);
  }
}

#[test]
fn test_get_value_returns_sequence_for_array() {
  let document = parse(indoc! {"
    tags:
      - ruby
      - rust
      - yaml
  "});

  let value = document.get_value("tags").unwrap();

  if let serde_yaml::Value::Sequence(seq) = value {
    assert_eq!(seq.len(), 3);
    assert_eq!(seq[0], serde_yaml::Value::String("ruby".to_string()));
    assert_eq!(seq[1], serde_yaml::Value::String("rust".to_string()));
    assert_eq!(seq[2], serde_yaml::Value::String("yaml".to_string()));
  } else {
    panic!("Expected Sequence, got: {:?}", value);
  }
}

#[test]
fn test_get_value_returns_full_object_at_index() {
  let document = parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  let value = document.get_value("[0]").unwrap();

  if let serde_yaml::Value::Mapping(map) = value {
    assert_eq!(
      map.get(&serde_yaml::Value::String("id".to_string())),
      Some(&serde_yaml::Value::String("talk-1".to_string()))
    );
    assert_eq!(
      map.get(&serde_yaml::Value::String("title".to_string())),
      Some(&serde_yaml::Value::String("First".to_string()))
    );
  } else {
    panic!("Expected Mapping, got: {:?}", value);
  }
}

#[test]
fn test_get_values_returns_objects_for_speakers() {
  let document = parse(indoc! {"
    - id: talk-1
      speakers:
        - name: Alice
          slug: alice
        - name: Bob
          slug: bob
    - id: talk-2
      speakers:
        - name: Charlie
          slug: charlie
  "});

  let values = document.get_values("[].speakers");

  assert_eq!(values.len(), 2);

  if let serde_yaml::Value::Sequence(speakers) = &values[0] {
    assert_eq!(speakers.len(), 2);

    if let serde_yaml::Value::Mapping(alice) = &speakers[0] {
      assert_eq!(
        alice.get(&serde_yaml::Value::String("name".to_string())),
        Some(&serde_yaml::Value::String("Alice".to_string()))
      );
    } else {
      panic!("Expected Mapping for speaker");
    }
  } else {
    panic!("Expected Sequence for speakers, got: {:?}", values[0]);
  }
}

#[test]
fn test_get_values_flattened_speakers() {
  let document = parse(indoc! {"
    - id: talk-1
      speakers:
        - name: Alice
        - name: Bob
    - id: talk-2
      speakers:
        - name: Charlie
  "});

  let values = document.get_values("[].speakers[]");

  assert_eq!(values.len(), 3);

  if let serde_yaml::Value::Mapping(alice) = &values[0] {
    assert_eq!(
      alice.get(&serde_yaml::Value::String("name".to_string())),
      Some(&serde_yaml::Value::String("Alice".to_string()))
    );
  } else {
    panic!("Expected Mapping, got: {:?}", values[0]);
  }
}

#[test]
fn test_get_values_returns_scalars_for_field_path() {
  let document = parse(indoc! {"
    - id: talk-1
      title: First
    - id: talk-2
      title: Second
  "});

  let values = document.get_values("[].title");

  assert_eq!(values.len(), 2);
  assert_eq!(values[0], serde_yaml::Value::String("First".to_string()));
  assert_eq!(values[1], serde_yaml::Value::String("Second".to_string()));
}

#[test]
fn test_get_value_none_for_missing_path() {
  let document = parse("host: localhost\n");

  assert!(document.get_value("missing").is_none());
}

#[test]
fn test_get_value_boolean() {
  let document = parse(indoc! {"
    enabled: true
    disabled: false
  "});

  assert_eq!(document.get_value("enabled"), Some(serde_yaml::Value::Bool(true)));
  assert_eq!(document.get_value("disabled"), Some(serde_yaml::Value::Bool(false)));
}

#[test]
fn test_get_value_null() {
  let document = parse(indoc! {"
    empty: null
  "});

  assert_eq!(document.get_value("empty"), Some(serde_yaml::Value::Null));
}

#[test]
fn test_path_ref_tags_vs_tags_bracket() {
  use yerba::Selector;

  let tags = Selector::parse("tags");
  let tags_bracket = Selector::parse("tags[]");

  assert_ne!(tags, tags_bracket);
  assert!(!tags.has_brackets());
  assert!(tags_bracket.has_brackets());
  assert!(tags_bracket.ends_with_bracket());
}

#[test]
fn test_path_ref_relative_vs_absolute() {
  use yerba::Selector;

  let relative = Selector::parse(".title");
  let absolute = Selector::parse("title");

  assert!(relative.is_relative());
  assert!(absolute.is_absolute());
  assert_ne!(relative, absolute);

  assert_eq!(relative.to_selector_string(), "title");
  assert_eq!(absolute.to_selector_string(), "title");
}

#[test]
fn test_path_ref_condition_split() {
  use yerba::Selector;

  let selector = Selector::parse("[].speakers[]");
  let (container, rest) = selector.split_at_last_bracket();
  assert_eq!(container.to_selector_string(), "[].speakers[]");
  assert!(rest.is_empty());

  let selector = Selector::parse("[].speakers[].name");
  let (container, field) = selector.split_at_last_bracket();
  assert_eq!(container.to_selector_string(), "[].speakers[]");
  assert_eq!(field.to_selector_string(), "name");

  let selector = Selector::parse("[].speakers[].name");
  let (container, rest) = selector.split_at_first_bracket();
  assert_eq!(container.to_selector_string(), "[]");
  assert_eq!(rest.to_selector_string(), "speakers[].name");
}

#[test]
fn test_filter_with_contains() {
  let document = parse(indoc! {"
    - id: talk-1
      title: Ruby on Rails
      speakers:
        - Alice
    - id: talk-2
      title: Python Basics
      speakers:
        - Bob
    - id: talk-3
      title: Advanced Ruby
      speakers:
        - Charlie
  "});

  let values = document.filter("[]", ".title contains Ruby");
  assert_eq!(values.len(), 2);
}

#[test]
fn test_filter_extracts_full_objects() {
  let document = parse(indoc! {"
    - id: talk-1
      video_provider: youtube
      video_id: abc
    - id: talk-2
      video_provider: vimeo
      video_id: def
  "});

  let values = document.filter("[]", ".video_provider == youtube");
  assert_eq!(values.len(), 1);

  if let serde_yaml::Value::Mapping(map) = &values[0] {
    assert_eq!(
      map.get(&serde_yaml::Value::String("video_id".to_string())),
      Some(&serde_yaml::Value::String("abc".to_string()))
    );
  } else {
    panic!("Expected Mapping");
  }
}

#[test]
fn test_get_values_with_empty_path_returns_all_items() {
  let document = parse(indoc! {"
    - ruby
    - rust
    - yaml
  "});

  let values = document.get_values("[]");
  assert_eq!(values.len(), 3);
  assert_eq!(values[0], serde_yaml::Value::String("ruby".to_string()));
}

#[test]
fn test_select_field_simple_key() {
  let yaml: serde_yaml::Value = serde_yaml::from_str("id: talk-1\ntitle: First\n").unwrap();

  assert_eq!(
    yerba::json::resolve_select_field(&yaml, "title"),
    serde_json::Value::String("First".to_string())
  );
}

#[test]
fn test_select_field_dot_prefix() {
  let yaml: serde_yaml::Value = serde_yaml::from_str("id: talk-1\ntitle: First\n").unwrap();

  assert_eq!(
    yerba::json::resolve_select_field(&yaml, ".title"),
    serde_json::Value::String("First".to_string())
  );
}

#[test]
fn test_select_field_nested_array() {
  let yaml: serde_yaml::Value = serde_yaml::from_str("speakers:\n  - name: Alice\n  - name: Bob\n").unwrap();

  let result = yerba::json::resolve_select_field(&yaml, ".speakers[].name");

  assert_eq!(
    result,
    serde_json::Value::Array(vec![
      serde_json::Value::String("Alice".to_string()),
      serde_json::Value::String("Bob".to_string()),
    ])
  );
}

#[test]
fn test_select_field_array_index() {
  let yaml: serde_yaml::Value = serde_yaml::from_str("tags:\n  - ruby\n  - rust\n  - yaml\n").unwrap();

  assert_eq!(
    yerba::json::resolve_select_field(&yaml, ".tags[0]"),
    serde_json::Value::String("ruby".to_string())
  );

  assert_eq!(
    yerba::json::resolve_select_field(&yaml, ".tags[2]"),
    serde_json::Value::String("yaml".to_string())
  );
}

#[test]
fn test_select_field_all_items() {
  let yaml: serde_yaml::Value = serde_yaml::from_str("tags:\n  - ruby\n  - rust\n").unwrap();

  assert_eq!(
    yerba::json::resolve_select_field(&yaml, ".tags[]"),
    serde_json::Value::Array(vec![
      serde_json::Value::String("ruby".to_string()),
      serde_json::Value::String("rust".to_string()),
    ])
  );
}

#[test]
fn test_select_field_missing_key() {
  let yaml: serde_yaml::Value = serde_yaml::from_str("id: talk-1\n").unwrap();

  assert_eq!(yerba::json::resolve_select_field(&yaml, ".missing"), serde_json::Value::Null);
}

#[test]
fn test_select_field_key_returns_root_name() {
  assert_eq!(yerba::json::select_field_key("title"), "title");
  assert_eq!(yerba::json::select_field_key(".title"), "title");
  assert_eq!(yerba::json::select_field_key(".speakers[].name"), "speakers");
  assert_eq!(yerba::json::select_field_key("speakers[0]"), "speakers");
}

// --- resolve_selectors ---

#[test]
fn test_resolve_selectors_root_sequence() {
  let document = parse(indoc! {"
    - id: a
    - id: b
    - id: c
  "});

  assert_eq!(document.resolve_selectors("[]"), vec!["[0]", "[1]", "[2]"]);
}

#[test]
fn test_resolve_selectors_nested_field() {
  let document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
  "});

  assert_eq!(document.resolve_selectors("[].title"), vec!["[0].title", "[1].title"]);
}

#[test]
fn test_resolve_selectors_nested_sequence() {
  let document = parse(indoc! {"
    - id: a
      speakers:
        - Alice
        - Bob
    - id: b
      speakers:
        - Charlie
  "});

  assert_eq!(
    document.resolve_selectors("[].speakers[]"),
    vec!["[0].speakers[0]", "[0].speakers[1]", "[1].speakers[0]"]
  );
}

#[test]
fn test_resolve_selectors_triple_nesting() {
  let document = parse(indoc! {"
    - talks:
        - speakers:
            - Alice
            - Bob
        - speakers:
            - Charlie
  "});

  assert_eq!(
    document.resolve_selectors("[].talks[].speakers[]"),
    vec!["[0].talks[0].speakers[0]", "[0].talks[0].speakers[1]", "[0].talks[1].speakers[0]"]
  );
}

#[test]
fn test_resolve_selectors_non_wildcard() {
  let document = parse(indoc! {"
    database:
      host: localhost
  "});

  assert_eq!(document.resolve_selectors("database.host"), vec!["database.host"]);
}

#[test]
fn test_resolve_selectors_empty_sequence() {
  let document = parse(indoc! {"
    tags: []
  "});

  assert_eq!(document.resolve_selectors("tags[]"), Vec::<String>::new());
}

#[test]
fn test_resolve_selectors_missing_field_skipped() {
  let document = parse(indoc! {"
    - id: a
      title: First
    - id: b
    - id: c
      title: Third
  "});

  // Only items with title are resolved
  assert_eq!(document.resolve_selectors("[].title"), vec!["[0].title", "[2].title"]);
}

// --- get_all_located ---

#[test]
fn test_get_all_located_returns_values_with_lines() {
  let document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
  "});

  let results = document.get_all_located("[].title");

  assert_eq!(results.len(), 2);
  assert_eq!(results[0].text, Some("First".to_string()));
  assert_eq!(results[0].line, 2);
  assert_eq!(results[1].text, Some("Second".to_string()));
  assert_eq!(results[1].line, 4);
}

#[test]
fn test_get_all_located_returns_selectors() {
  let document = parse(indoc! {"
    - id: a
      speakers:
        - Alice
        - Bob
    - id: b
      speakers:
        - Charlie
  "});

  let results = document.get_all_located("[].speakers[]");

  assert_eq!(results.len(), 3);
  assert_eq!(results[0].text, Some("Alice".to_string()));
  assert_eq!(results[0].selector, "[0].speakers[0]");
  assert_eq!(results[1].text, Some("Bob".to_string()));
  assert_eq!(results[1].selector, "[0].speakers[1]");
  assert_eq!(results[2].text, Some("Charlie".to_string()));
  assert_eq!(results[2].selector, "[1].speakers[0]");
}

#[test]
fn test_get_all_located_scalars_have_text_and_type() {
  let document = parse(indoc! {"
    - name: Alice
    - name: Bob
  "});

  let results = document.get_all_located("[].name");

  assert_eq!(results.len(), 2);
  assert_eq!(results[0].text, Some("Alice".to_string()));
  assert_eq!(results[0].node_type, "scalar");
  assert!(results[0].value_type.is_some());
}

#[test]
fn test_get_all_located_maps_have_no_text() {
  let document = parse(indoc! {"
    - id: a
      title: First
    - id: b
      title: Second
  "});

  let results = document.get_all_located("[]");

  assert_eq!(results.len(), 2);
  assert_eq!(results[0].node_type, "map");
  assert_eq!(results[0].text, None);
  assert_eq!(results[0].selector, "[0]");
  assert_eq!(results[1].selector, "[1]");
}

#[test]
fn test_get_all_located_sequences_have_no_text() {
  let document = parse(indoc! {"
    - id: a
      speakers:
        - Alice
    - id: b
      speakers:
        - Bob
  "});

  let results = document.get_all_located("[].speakers");

  assert_eq!(results.len(), 2);
  assert_eq!(results[0].node_type, "sequence");
  assert_eq!(results[0].text, None);
  assert_eq!(results[0].selector, "[0].speakers");
  assert_eq!(results[1].selector, "[1].speakers");
}

#[test]
fn test_get_all_located_lines_are_correct() {
  let document = parse(indoc! {"
    host: localhost
    port: 5432
    name: mydb
  "});

  let results = document.get_all_located("port");

  assert_eq!(results.len(), 1);
  assert_eq!(results[0].text, Some("5432".to_string()));
  assert_eq!(results[0].line, 2);
  assert_eq!(results[0].selector, "port");
}

#[test]
fn test_get_all_located_with_file_path() {
  use std::io::Write;

  let mut file = tempfile::NamedTempFile::new().unwrap();
  writeln!(file, "name: Alice").unwrap();

  let document = yerba::Document::parse_file(file.path()).unwrap();
  let results = document.get_all_located("name");

  assert_eq!(results.len(), 1);
  assert_eq!(results[0].text, Some("Alice".to_string()));
  assert!(results[0].file_path.is_some());
  assert!(!results[0].file_path.as_ref().unwrap().is_empty());
}
