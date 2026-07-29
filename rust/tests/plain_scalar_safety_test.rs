mod support;
use indoc::indoc;
use support::parse;
use yerba::{is_plain_safe, is_plain_safe_in_flow, is_quoted_scalar, is_valid_inline_value, needs_quoting, quote_if_needed, InsertPosition};

#[test]
fn test_plain_safe_accepts_ordinary_strings() {
  assert!(is_plain_safe("plain text"));
  assert!(is_plain_safe("Why Git Still Matters"));
  assert!(is_plain_safe("2026-03-26"));
  assert!(is_plain_safe("a-b-c"));
  assert!(is_plain_safe("has#hash"));
  assert!(is_plain_safe("ratio 1:2"));
}

#[test]
fn test_plain_safe_rejects_mapping_indicators() {
  assert!(!is_plain_safe("One Record: Concurrency"));
  assert!(!is_plain_safe("trailing colon:"));
  assert!(!is_plain_safe(":"));
}

#[test]
fn test_plain_safe_rejects_comment_indicators() {
  assert!(!is_plain_safe("#hashstart"));
  assert!(!is_plain_safe("has # hash"));
}

#[test]
fn test_plain_safe_rejects_leading_indicators() {
  for value in [
    "&anchor",
    "*alias",
    "!tag",
    "|literal",
    ">folded",
    "'single",
    "\"double",
    "%directive",
    "@reserved",
    "`backtick",
    ",comma",
    "[bracket",
    "]bracket",
    "{brace",
    "}brace",
  ] {
    assert!(!is_plain_safe(value), "expected {:?} to be unsafe plain", value);
  }
}

#[test]
fn test_plain_safe_rejects_sequence_and_document_markers() {
  assert!(!is_plain_safe("- leading dash"));
  assert!(!is_plain_safe("? question"));
  assert!(!is_plain_safe("-"));
  assert!(!is_plain_safe("---"));
  assert!(!is_plain_safe("--- doc"));
  assert!(!is_plain_safe("... end"));
}

#[test]
fn test_plain_safe_allows_hyphen_when_not_an_indicator() {
  assert!(is_plain_safe("-dZZJ6pex-g"));
  assert!(is_plain_safe("-42"));
}

#[test]
fn test_plain_safe_rejects_whitespace_edges_and_breaks() {
  assert!(!is_plain_safe(""));
  assert!(!is_plain_safe("  padded  "));
  assert!(!is_plain_safe("trailing "));
  assert!(!is_plain_safe("two\nlines"));
  assert!(!is_plain_safe("tab\there"));
}

#[test]
fn test_plain_safe_in_flow_is_stricter() {
  assert!(is_plain_safe("a, b"));
  assert!(!is_plain_safe_in_flow("a, b"));

  assert!(is_plain_safe("ends]"));
  assert!(!is_plain_safe_in_flow("ends]"));

  assert!(is_plain_safe_in_flow("plain"));
}

#[test]
fn test_needs_quoting_covers_type_and_structure() {
  assert!(needs_quoting("true"));
  assert!(needs_quoting("null"));
  assert!(needs_quoting("12"));
  assert!(needs_quoting("One Record: Concurrency"));
  assert!(!needs_quoting("plain text"));
}

#[test]
fn test_quote_if_needed_escapes_backslashes_and_quotes() {
  assert_eq!(quote_if_needed("a: b"), "\"a: b\"");
  assert_eq!(quote_if_needed("say \"hi\": now"), "\"say \\\"hi\\\": now\"");
  assert_eq!(quote_if_needed("back\\slash: x"), "\"back\\\\slash: x\"");
  assert_eq!(quote_if_needed("plain"), "plain");
}

#[test]
fn test_is_quoted_scalar() {
  assert!(is_quoted_scalar("\"a: b\""));
  assert!(is_quoted_scalar("'a: b'"));
  assert!(is_quoted_scalar("\"escaped \\\" quote\""));
  assert!(is_quoted_scalar("'doubled '' quote'"));

  assert!(!is_quoted_scalar("plain"));
  assert!(!is_quoted_scalar("\"unterminated"));

  assert!(!is_quoted_scalar("\"a\" and \"b\""));
  assert!(!is_quoted_scalar("'a' and 'b'"));
  assert!(!is_quoted_scalar("\""));
}

#[test]
fn test_is_valid_inline_value_keeps_raw_yaml_text_intact() {
  assert!(is_valid_inline_value("plain"));
  assert!(is_valid_inline_value("\"quoted: text\""));
  assert!(is_valid_inline_value("[]"));
  assert!(is_valid_inline_value("[a, b]"));
  assert!(is_valid_inline_value("{}"));
  assert!(is_valid_inline_value("{k: v}"));
  assert!(is_valid_inline_value("host: localhost\nport: 5432"));
  assert!(is_valid_inline_value("- ruby\n- rails"));
  // A single-entry block sequence is still a fragment, not a string.
  assert!(is_valid_inline_value("- new"));

  assert!(is_valid_inline_value(""));

  assert!(!is_valid_inline_value("One Record: Concurrency"));
  assert!(!is_valid_inline_value("#hashstart"));
  assert!(!is_valid_inline_value("  padded  "));
}

#[test]
fn test_insert_map_key_quotes_value_containing_colon() {
  let mut document = parse(indoc! {"
    talk:
      title: Two Requests
  "});

  document.insert_into("talk.raw_title", "One Record: Concurrency", InsertPosition::Last).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      talk:
        title: Two Requests
        raw_title: \"One Record: Concurrency\"
    "}
  );
}

#[test]
fn test_insert_map_key_round_trips_hazardous_values() {
  for value in [
    "One Record: Concurrency",
    "#hashstart",
    "has # hash",
    "trailing colon:",
    "  padded  ",
    "*alias",
    "?  question",
    "a, b",
    "back\\slash: x",
  ] {
    let mut document = parse("root:\n  existing: 1\n");

    document.insert_into("root.added", value, InsertPosition::Last).unwrap();

    assert_eq!(
      document.get_value("root.added"),
      Some(yaml_serde::Value::String(value.to_string())),
      "value {:?} did not round-trip; document was:\n{}",
      value,
      document
    );
  }
}

#[test]
fn test_insert_map_key_leaves_safe_values_plain() {
  let mut document = parse("root:\n  existing: 1\n");

  document.insert_into("root.added", "Why Git Still Matters", InsertPosition::Last).unwrap();

  assert_eq!(document.to_string(), "root:\n  existing: 1\n  added: Why Git Still Matters\n");
}

#[test]
fn test_insert_map_key_does_not_requote_an_already_quoted_value() {
  let mut document = parse("root:\n  existing: 1\n");

  document.insert_into("root.added", "\"One Record: Concurrency\"", InsertPosition::Last).unwrap();

  assert_eq!(document.to_string(), "root:\n  existing: 1\n  added: \"One Record: Concurrency\"\n");
  assert_eq!(
    document.get_value("root.added"),
    Some(yaml_serde::Value::String("One Record: Concurrency".to_string()))
  );
}

#[test]
fn test_insert_map_key_preserves_multi_line_block_values() {
  let mut document = parse("root:\n  existing: 1\n");

  document
    .insert_into("root.nested", "host: localhost\nport: 5432", InsertPosition::Last)
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      root:
        existing: 1
        nested:
          host: localhost
          port: 5432
    "}
  );
}

#[test]
fn test_insert_map_key_treats_dash_prefixed_values_as_a_nested_sequence() {
  let mut document = parse("root:\n  existing: 1\n");

  document.insert_into("root.added", "- leading dash", InsertPosition::Last).unwrap();

  assert_eq!(
    document.get_value("root.added"),
    Some(yaml_serde::Value::Sequence(vec![yaml_serde::Value::String("leading dash".to_string())]))
  );

  let mut quoted = parse("root:\n  existing: 1\n");

  quoted.insert_into("root.added", "\"- leading dash\"", InsertPosition::Last).unwrap();

  assert_eq!(quoted.get_value("root.added"), Some(yaml_serde::Value::String("- leading dash".to_string())));
}

#[test]
fn test_insert_sequence_item_still_accepts_yaml_fragments() {
  let mut document = parse("- name: \"Alice\"\n");

  document.insert_into("", "name: Bob", InsertPosition::Last).unwrap();

  assert_eq!(document.to_string(), "- name: \"Alice\"\n- name: Bob\n");
}

#[test]
fn test_set_promotes_plain_target_when_value_is_unsafe() {
  let mut document = parse("a: plainvalue\n");

  document.set("a", "One Record: Concurrency").unwrap();

  assert_eq!(document.to_string(), "a: \"One Record: Concurrency\"\n");
  assert_eq!(document.get_value("a"), Some(yaml_serde::Value::String("One Record: Concurrency".to_string())));
}

#[test]
fn test_set_keeps_plain_target_plain_for_safe_values() {
  let mut document = parse("a: plainvalue\n");

  document.set("a", "Why Git Still Matters").unwrap();

  assert_eq!(document.to_string(), "a: Why Git Still Matters\n");
}

#[test]
fn test_set_still_preserves_existing_quote_style() {
  let mut document = parse("a: \"quoted\"\nb: 'single'\n");

  document.set("a", "One Record: Concurrency").unwrap();
  document.set("b", "One Record: Concurrency").unwrap();

  assert_eq!(document.to_string(), "a: \"One Record: Concurrency\"\nb: 'One Record: Concurrency'\n");
}

#[test]
fn test_set_all_promotes_plain_targets() {
  let mut document = parse("items:\n  - name: one\n  - name: two\n");

  document.set_all("items[].name", "a: b").unwrap();

  assert_eq!(document.to_string(), "items:\n  - name: \"a: b\"\n  - name: \"a: b\"\n");
}

#[test]
fn test_set_leaves_assembled_yaml_text_alone() {
  let mut flow = parse("name: Alice\ntags: old\n");

  flow.set("tags", "[]").unwrap();

  assert_eq!(flow.to_string(), "name: Alice\ntags: []\n");

  let mut sequence = parse("name: Alice\ntags: old\n");

  sequence.set("tags", "- new").unwrap();

  assert_eq!(sequence.to_string(), "name: Alice\ntags: - new\n");

  let mut quoted = parse("name: Alice\ntags: old\n");

  quoted.set("tags", "\"a: b\"").unwrap();

  assert_eq!(quoted.to_string(), "name: Alice\ntags: \"a: b\"\n");
}
