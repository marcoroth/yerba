mod support;
use support::parse;
use yerba::{escape_double_quoted, is_control_character, is_plain_safe};

#[test]
fn test_control_characters_are_the_set_libyaml_rejects() {
  assert!(is_control_character('\u{0}'));
  assert!(is_control_character('\u{8}'));
  assert!(is_control_character('\u{b}'));
  assert!(is_control_character('\u{1b}'));
  assert!(is_control_character('\u{7f}'));
  assert!(is_control_character('\u{80}'));
  assert!(is_control_character('\u{9f}'));
}

#[test]
fn test_line_breaks_and_tabs_are_not_control_characters() {
  assert!(!is_control_character('\t'));
  assert!(!is_control_character('\n'));
  assert!(!is_control_character('\r'));

  // libyaml reads NEL as a line break rather than rejecting it.
  assert!(!is_control_character('\u{85}'));

  assert!(!is_control_character('\u{a0}'));
  assert!(!is_control_character('\u{feff}'));
  assert!(!is_control_character('a'));
}

#[test]
fn test_plain_safe_rejects_control_characters() {
  assert!(!is_plain_safe("abc\u{8}def"));
  assert!(!is_plain_safe("abc\u{7f}def"));
  assert!(is_plain_safe("abc def"));
}

#[test]
fn test_escape_double_quoted_escapes_line_breaks_and_control_characters() {
  assert_eq!(escape_double_quoted("line one\nline two"), "line one\\nline two");
  assert_eq!(escape_double_quoted("a\rb"), "a\\rb");
  assert_eq!(escape_double_quoted("a\tb"), "a\\tb");
  assert_eq!(escape_double_quoted("abc\u{8}def"), "abc\\x08def");
  assert_eq!(escape_double_quoted("esc\u{1b}[0m"), "esc\\x1b[0m");
  assert_eq!(escape_double_quoted("say \"hi\""), "say \\\"hi\\\"");
  assert_eq!(escape_double_quoted("back\\slash"), "back\\\\slash");
  assert_eq!(escape_double_quoted("plain"), "plain");
}

#[test]
fn test_setting_a_value_with_a_control_character_escapes_it() {
  let mut document = parse("title: Hello\n");

  document.set("title", "abc\u{8}def").unwrap();

  assert_eq!(document.to_string(), "title: \"abc\\x08def\"\n");
}

#[test]
fn test_a_multiline_value_round_trips_through_an_escaped_scalar() {
  let mut document = parse("note: old\n");

  document.set("note", "line one\nline two").unwrap();

  assert_eq!(document.to_string(), "note: \"line one\\nline two\"\n");
  assert_eq!(parse(&document.to_string()).get("note").unwrap(), "line one\nline two");
}

#[test]
fn test_hex_escapes_are_decoded_on_read() {
  let document = parse("a: \"abc\\x08def\"\nb: \"\\u00e9\"\nc: \"\\U0001F600\"\n");

  assert_eq!(document.get("a").unwrap(), "abc\u{8}def");
  assert_eq!(document.get("b").unwrap(), "é");
  assert_eq!(document.get("c").unwrap(), "😀");
}

#[test]
fn test_malformed_hex_escapes_are_left_alone() {
  let document = parse("a: \"\\xZZ\"\nb: \"\\u12\"\n");

  assert_eq!(document.get("a").unwrap(), "\\xZZ");
  assert_eq!(document.get("b").unwrap(), "\\u12");
}
