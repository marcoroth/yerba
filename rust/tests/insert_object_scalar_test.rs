use yerba::{json_to_yaml_text, QuoteStyle};

#[test]
fn multiline_strings_are_escaped_rather_than_broken_across_lines() {
  let value = serde_json::json!({ "desc": "line one\nline two" });
  let yaml = json_to_yaml_text(&value, &QuoteStyle::Double, 0);

  assert_eq!(yaml, "desc: \"line one\\nline two\"");
}

#[test]
fn tabs_and_control_characters_are_escaped() {
  let value = serde_json::json!({ "desc": "a\tb\u{7}c" });
  let yaml = json_to_yaml_text(&value, &QuoteStyle::Double, 0);

  assert_eq!(yaml, "desc: \"a\\tb\\x07c\"");
}

#[test]
fn an_empty_sequence_is_written_inline() {
  let value = serde_json::json!({ "speakers": [], "date": "x" });
  let yaml = json_to_yaml_text(&value, &QuoteStyle::Double, 0);

  assert_eq!(yaml, "speakers: []\ndate: \"x\"");
}

#[test]
fn an_empty_mapping_is_written_inline() {
  let value = serde_json::json!({ "meta": {}, "date": "x" });
  let yaml = json_to_yaml_text(&value, &QuoteStyle::Double, 0);

  assert_eq!(yaml, "meta: {}\ndate: \"x\"");
}

#[test]
fn a_multiline_string_cannot_be_single_quoted() {
  let value = serde_json::json!({ "desc": "line one\nline two" });
  let yaml = json_to_yaml_text(&value, &QuoteStyle::Single, 0);

  assert_eq!(yaml, "desc: \"line one\\nline two\"");
}
