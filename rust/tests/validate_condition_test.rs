mod support;
use indoc::indoc;
use support::parse;
use yerba::{validate_condition, validate_item_condition};

#[test]
fn test_valid_conditions_are_accepted() {
  for condition in [
    ".kind == keynote",
    ".kind != keynote",
    ".title contains Ruby",
    ".title not_contains test",
    ".venue.city == Berlin",
    ".speakers[].name == Alice",
    ".kind == \"quoted value\"",
    "  .kind == keynote  ",
  ] {
    assert!(validate_condition(condition).is_ok(), "{}", condition);
    assert!(validate_item_condition(condition).is_ok(), "{}", condition);
  }
}

#[test]
fn test_unparseable_conditions_are_rejected() {
  for condition in ["", "   ", "garbage", "garbage!!", ".kind = keynote", "keynote"] {
    assert!(validate_condition(condition).is_err(), "{}", condition);
  }
}

#[test]
fn test_triple_equals_parses_as_a_value_starting_with_equals() {
  assert!(validate_condition(".kind === keynote").is_ok());
}

#[test]
fn test_missing_operator_reports_the_operators() {
  let error = validate_condition("garbage!!").unwrap_err().to_string();

  assert!(error.contains("not_contains"), "{}", error);
  assert!(error.contains("garbage!!"), "{}", error);
}

#[test]
fn test_empty_condition_is_reported_as_empty() {
  assert!(validate_condition("").unwrap_err().to_string().contains("empty"));
}

#[test]
fn test_condition_without_a_selector_is_rejected() {
  let error = validate_condition(". == keynote").unwrap_err().to_string();

  assert!(error.contains("missing a selector"), "{}", error);
}

#[test]
fn test_absolute_selector_is_allowed_without_item_context() {
  assert!(validate_condition("kind == keynote").is_ok());
  assert!(validate_condition("database.host == localhost").is_ok());
}

#[test]
fn test_absolute_selector_is_rejected_with_item_context() {
  let error = validate_item_condition("kind == keynote").unwrap_err().to_string();

  assert!(error.contains("must be relative"), "{}", error);
  assert!(error.contains("did you mean \".kind\""), "{}", error);
}

#[test]
fn test_filter_rejects_a_malformed_condition() {
  let document = parse(indoc! {"
    - kind: keynote
    - kind: talk
  "});

  assert!(document.filter("[]", ".kind = keynote").is_err());
  assert!(document.filter("[]", "kind == keynote").is_err());
  assert!(document.filter("[]", "").is_err());

  assert_eq!(document.filter("[]", ".kind == keynote").unwrap().len(), 1);
}

#[test]
fn test_filter_with_selectors_rejects_a_malformed_condition() {
  let document = parse(indoc! {"
    - kind: keynote
    - kind: talk
  "});

  assert!(document.filter_with_selectors("[]", "kind == keynote").is_err());
  assert_eq!(document.filter_with_selectors("[]", ".kind == keynote").unwrap().len(), 1);
}

#[test]
fn test_find_items_rejects_a_malformed_condition() {
  let document = parse(indoc! {"
    - kind: keynote
    - kind: talk
  "});

  assert!(document.find_items("[]", Some("kind == keynote"), None).is_err());
  assert_eq!(document.find_items("[]", Some(".kind == keynote"), None).unwrap().len(), 1);
}

#[test]
fn test_a_malformed_condition_does_not_read_as_no_match() {
  let document = parse(indoc! {"
    - kind: keynote
    - kind: talk
  "});

  let genuine_no_match = document.filter("[]", ".kind == workshop").unwrap();

  assert!(genuine_no_match.is_empty());
  assert!(document.filter("[]", ".kind = workshop").is_err());
}
