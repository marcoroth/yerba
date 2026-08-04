mod support;
use support::parse;

fn value(source: &str) -> String {
  parse(source).get("k").unwrap()
}

#[test]
fn test_a_wrapped_double_quoted_scalar_folds_to_a_space() {
  assert_eq!(value("k: \"a\n  b\"\n"), "a b");
  assert_eq!(value("k: \"a\n  b\n  c\"\n"), "a b c");
}

#[test]
fn test_a_blank_line_in_a_double_quoted_scalar_reads_as_a_newline() {
  assert_eq!(value("k: \"a\n\n  b\"\n"), "a\nb");
  assert_eq!(value("k: \"a\n\n\n  b\"\n"), "a\n\nb");
}

#[test]
fn test_an_escaped_break_joins_the_lines_with_nothing() {
  assert_eq!(value("k: \"a \\\n  b\"\n"), "a b");
  assert_eq!(value("k: \"ab\\\n  cd\"\n"), "abcd");
}

#[test]
fn test_an_escaped_newline_is_not_folded() {
  assert_eq!(value("k: \"a\\nb\"\n"), "a\nb");
  assert_eq!(value("k: \"a\\nb\n  c\"\n"), "a\nb c");
}

#[test]
fn test_folding_keeps_whitespace_at_the_edges_of_the_scalar() {
  assert_eq!(value("k: \"  a\n  b\"\n"), "  a b");
  assert_eq!(value("k: \"a\n  b  \"\n"), "a b  ");
}

#[test]
fn test_a_wrapped_single_quoted_scalar_folds() {
  assert_eq!(value("k: 'a\n  b'\n"), "a b");
  assert_eq!(value("k: 'a\n\n  b'\n"), "a\nb");
  assert_eq!(value("k: 'a''b\n  c'\n"), "a'b c");
}

#[test]
fn test_a_wrapped_plain_scalar_folds() {
  assert_eq!(value("k: a\n  b\n"), "a b");
}

#[test]
fn test_a_literal_block_scalar_keeps_its_breaks() {
  assert_eq!(value("k: |-\n  a\n  b\n"), "a\nb");
}

#[test]
fn test_a_folded_block_scalar_folds_its_breaks() {
  assert_eq!(value("k: >-\n  a\n  b\n"), "a b");
}

#[test]
fn test_a_wrapped_value_survives_being_written_back() {
  let mut document = parse("k: \"a\n  b\"\n");
  let read = document.get("k").unwrap();

  document.set("k", &read).unwrap();

  assert_eq!(document.to_string(), "k: \"a b\"\n");
  assert_eq!(parse(&document.to_string()).get("k").unwrap(), read);
}

#[test]
fn test_an_escape_in_a_wrapped_value_survives_being_written_back() {
  let mut document = parse("k: \"a \\U0001F971 b\n  c\"\n");
  let read = document.get("k").unwrap();

  assert_eq!(read, "a 🥱 b c");

  document.set("k", &read).unwrap();

  assert_eq!(parse(&document.to_string()).get("k").unwrap(), read);
}
