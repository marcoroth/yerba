use yerba::selector::{Selector, SelectorSegment};

#[test]
fn test_parse_empty() {
  let path = Selector::parse("");

  assert_eq!(path, Selector::Absolute(vec![]));
  assert!(path.is_empty());
}

#[test]
fn test_parse_simple_key() {
  let path = Selector::parse("host");

  assert_eq!(path, Selector::Absolute(vec![SelectorSegment::Key("host".to_string())]));
  assert!(path.is_absolute());
}

#[test]
fn test_parse_nested_key() {
  let path = Selector::parse("database.host");

  assert_eq!(
    path,
    Selector::Absolute(vec![SelectorSegment::Key("database".to_string()), SelectorSegment::Key("host".to_string()),])
  );
}

#[test]
fn test_parse_relative_key() {
  let path = Selector::parse(".title");

  assert_eq!(path, Selector::Relative(vec![SelectorSegment::Key("title".to_string())]));
  assert!(path.is_relative());
}

#[test]
fn test_parse_relative_nested() {
  let path = Selector::parse(".speakers[].name");

  assert_eq!(
    path,
    Selector::Relative(vec![
      SelectorSegment::Key("speakers".to_string()),
      SelectorSegment::AllItems,
      SelectorSegment::Key("name".to_string()),
    ])
  );
}

#[test]
fn test_parse_all_items() {
  let path = Selector::parse("[]");

  assert_eq!(path, Selector::Absolute(vec![SelectorSegment::AllItems]));
  assert!(path.ends_with_bracket());
}

#[test]
fn test_parse_index() {
  let path = Selector::parse("[0]");

  assert_eq!(path, Selector::Absolute(vec![SelectorSegment::Index(0)]));
  assert!(path.ends_with_bracket());
}

#[test]
fn test_parse_bracket_with_field() {
  let path = Selector::parse("[].title");

  assert_eq!(
    path,
    Selector::Absolute(vec![SelectorSegment::AllItems, SelectorSegment::Key("title".to_string())])
  );

  assert!(!path.ends_with_bracket());
}

#[test]
fn test_parse_index_with_field() {
  let path = Selector::parse("[0].title");

  assert_eq!(
    path,
    Selector::Absolute(vec![SelectorSegment::Index(0), SelectorSegment::Key("title".to_string())])
  );
}

#[test]
fn test_parse_nested_brackets() {
  let path = Selector::parse("[].speakers[].name");

  assert_eq!(
    path,
    Selector::Absolute(vec![
      SelectorSegment::AllItems,
      SelectorSegment::Key("speakers".to_string()),
      SelectorSegment::AllItems,
      SelectorSegment::Key("name".to_string()),
    ])
  );
}

#[test]
fn test_parse_nested_brackets_no_trailing_field() {
  let path = Selector::parse("[].speakers[]");

  assert_eq!(
    path,
    Selector::Absolute(vec![
      SelectorSegment::AllItems,
      SelectorSegment::Key("speakers".to_string()),
      SelectorSegment::AllItems,
    ])
  );

  assert!(path.ends_with_bracket());
}

#[test]
fn test_parse_deep_nesting() {
  let path = Selector::parse("[].talks[].speakers[].name");

  assert_eq!(
    path,
    Selector::Absolute(vec![
      SelectorSegment::AllItems,
      SelectorSegment::Key("talks".to_string()),
      SelectorSegment::AllItems,
      SelectorSegment::Key("speakers".to_string()),
      SelectorSegment::AllItems,
      SelectorSegment::Key("name".to_string()),
    ])
  );
}

#[test]
fn test_parse_key_then_bracket() {
  let path = Selector::parse("tags.[]");

  assert_eq!(
    path,
    Selector::Absolute(vec![SelectorSegment::Key("tags".to_string()), SelectorSegment::AllItems])
  );
}

#[test]
fn test_relative_vs_absolute() {
  assert!(Selector::parse(".title").is_relative());
  assert!(Selector::parse(".speakers[].name").is_relative());
  assert!(Selector::parse("title").is_absolute());
  assert!(Selector::parse("[].title").is_absolute());
  assert!(Selector::parse("database.host").is_absolute());
  assert!(Selector::parse("[]").is_absolute());
  assert!(Selector::parse("[0]").is_absolute());
}

#[test]
fn test_has_brackets() {
  assert!(!Selector::parse("database.host").has_brackets());
  assert!(!Selector::parse(".title").has_brackets());
  assert!(Selector::parse("[]").has_brackets());
  assert!(Selector::parse("[0]").has_brackets());
  assert!(Selector::parse("[].title").has_brackets());
  assert!(Selector::parse(".speakers[]").has_brackets());
}

#[test]
fn test_split_at_last_bracket_simple() {
  let path = Selector::parse("[].title");
  let (container, field) = path.split_at_last_bracket();

  assert_eq!(container.to_selector_string(), "[]");
  assert_eq!(field.to_selector_string(), "title");
  assert!(field.is_relative());
}

#[test]
fn test_split_at_last_bracket_nested() {
  let path = Selector::parse("[].speakers[].name");
  let (container, field) = path.split_at_last_bracket();

  assert_eq!(container.to_selector_string(), "[].speakers[]");
  assert_eq!(field.to_selector_string(), "name");
  assert!(field.is_relative());
}

#[test]
fn test_split_at_last_bracket_no_field() {
  let path = Selector::parse("[]");
  let (container, field) = path.split_at_last_bracket();

  assert_eq!(container.to_selector_string(), "[]");
  assert!(field.is_empty());
}

#[test]
fn test_split_at_last_bracket_ends_with_bracket() {
  let path = Selector::parse("[].speakers[]");
  let (container, field) = path.split_at_last_bracket();

  assert_eq!(container.to_selector_string(), "[].speakers[]");
  assert!(field.is_empty());
}

#[test]
fn test_split_at_last_bracket_no_brackets() {
  let path = Selector::parse("database.host");
  let (container, field) = path.split_at_last_bracket();

  assert!(container.is_empty());
  assert_eq!(field.to_selector_string(), "database.host");
}

#[test]
fn test_split_at_first_bracket_simple() {
  let path = Selector::parse("[].title");
  let (container, rest) = path.split_at_first_bracket();

  assert_eq!(container.to_selector_string(), "[]");
  assert_eq!(rest.to_selector_string(), "title");
}

#[test]
fn test_split_at_first_bracket_nested() {
  let path = Selector::parse("[].speakers[].name");
  let (container, rest) = path.split_at_first_bracket();

  assert_eq!(container.to_selector_string(), "[]");
  assert_eq!(rest.to_selector_string(), "speakers[].name");
}

#[test]
fn test_split_at_first_bracket_speakers_only() {
  let path = Selector::parse("[].speakers[]");
  let (container, rest) = path.split_at_first_bracket();

  assert_eq!(container.to_selector_string(), "[]");
  assert_eq!(rest.to_selector_string(), "speakers[]");
}

#[test]
fn test_split_at_first_bracket_just_bracket() {
  let path = Selector::parse("[]");
  let (container, rest) = path.split_at_first_bracket();

  assert_eq!(container.to_selector_string(), "[]");
  assert!(rest.is_empty());
}

#[test]
fn test_to_selector_string_roundtrip() {
  let cases = vec![
    "",
    "host",
    "database.host",
    "[]",
    "[0]",
    "[].title",
    "[0].title",
    "[].speakers[]",
    "[].speakers[].name",
    "[].talks[].speakers[].name",
  ];

  for input in cases {
    let path = Selector::parse(input);
    assert_eq!(path.to_selector_string(), input, "roundtrip failed for: {}", input);
  }
}

#[test]
fn test_to_selector_string_normalizes_dot_bracket() {
  let path = Selector::parse("tags.[]");
  assert_eq!(path.to_selector_string(), "tags[]");
}

#[test]
fn test_to_selector_string_relative() {
  let path = Selector::parse(".title");
  assert_eq!(path.to_selector_string(), "title");
}

#[test]
fn test_to_selector_string_relative_nested() {
  let path = Selector::parse(".speakers[].name");
  assert_eq!(path.to_selector_string(), "speakers[].name");
}

// --- resolve_relative ---

#[test]
fn test_resolve_relative_simple() {
  let relative = Selector::parse(".title");
  let base = Selector::parse("[]");
  let resolved = relative.resolve_relative(&base);

  assert!(resolved.is_absolute());
  assert_eq!(resolved.to_selector_string(), "[].title");
}

#[test]
fn test_resolve_relative_nested_base() {
  let relative = Selector::parse(".name");
  let base = Selector::parse("[].speakers[]");
  let resolved = relative.resolve_relative(&base);

  assert_eq!(resolved.to_selector_string(), "[].speakers[].name");
}

#[test]
fn test_resolve_relative_with_brackets() {
  let relative = Selector::parse(".speakers[]");
  let base = Selector::parse("[]");
  let resolved = relative.resolve_relative(&base);

  assert_eq!(resolved.to_selector_string(), "[].speakers[]");
}

#[test]
fn test_resolve_absolute_unchanged() {
  let absolute = Selector::parse("[].title");
  let base = Selector::parse("[].speakers[]");
  let resolved = absolute.resolve_relative(&base);

  assert_eq!(resolved.to_selector_string(), "[].title");
}

#[test]
fn test_resolve_relative_empty_base() {
  let relative = Selector::parse(".title");
  let base = Selector::parse("");
  let resolved = relative.resolve_relative(&base);

  assert_eq!(resolved.to_selector_string(), "title");
}

#[test]
fn test_parse_all_keys() {
  let path = Selector::parse("*");

  assert_eq!(path, Selector::Absolute(vec![SelectorSegment::AllKeys]));
  assert!(path.has_wildcard());
}

#[test]
fn test_parse_nested_all_keys() {
  let path = Selector::parse("venue.*");

  assert_eq!(
    path,
    Selector::Absolute(vec![SelectorSegment::Key("venue".to_string()), SelectorSegment::AllKeys])
  );
}

#[test]
fn test_parse_all_keys_after_all_items() {
  let path = Selector::parse("[].*");

  assert_eq!(path, Selector::Absolute(vec![SelectorSegment::AllItems, SelectorSegment::AllKeys]));
}

#[test]
fn test_parse_key_after_all_keys() {
  let path = Selector::parse("*.city");

  assert_eq!(
    path,
    Selector::Absolute(vec![SelectorSegment::AllKeys, SelectorSegment::Key("city".to_string())])
  );
}

#[test]
fn test_all_keys_round_trips_to_selector_string() {
  for input in ["*", "venue.*", "[].*", "*.city", "[0].venue.*"] {
    assert_eq!(Selector::parse(input).to_selector_string(), input);
  }
}

#[test]
fn test_all_keys_counts_as_a_wildcard() {
  assert!(Selector::parse("venue.*").has_wildcard());
  assert!(Selector::parse("[].title").has_wildcard());
  assert!(!Selector::parse("venue.city").has_wildcard());
}

#[test]
fn test_all_keys_is_not_a_bracket() {
  let path = Selector::parse("venue.*");

  assert!(!path.has_brackets());
  assert!(!path.ends_with_bracket());
}
