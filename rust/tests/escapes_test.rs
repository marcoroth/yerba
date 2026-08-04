mod support;
use support::parse;

#[test]
fn test_decode_escapes_spells_characters_the_style_can_carry() {
  let mut document = parse("k: \"a \\U0001F971 b\"\n");

  let count = document.decode_escapes(None).unwrap();

  assert_eq!(count, 1);
  assert_eq!(document.to_string(), "k: \"a 🥱 b\"\n");
}

#[test]
fn test_decode_escapes_keeps_escapes_that_have_to_stay() {
  let mut document = parse("k: \"keep\\x08this\"\nl: \"a\\nb\"\nm: \"say \\\"hi\\\"\"\n");

  document.decode_escapes(None).unwrap();

  assert_eq!(document.to_string(), "k: \"keep\\x08this\"\nl: \"a\\nb\"\nm: \"say \\\"hi\\\"\"\n");
}

#[test]
fn test_decode_escapes_writes_a_wrapped_value_on_one_line() {
  let mut document = parse("k: \"a\n  b\"\n");

  document.decode_escapes(None).unwrap();

  assert_eq!(document.to_string(), "k: \"a b\"\n");
}

#[test]
fn test_decode_escapes_leaves_other_styles_alone() {
  let source = "k: plain\nl: 'single'\nm: |-\n  block\n";
  let mut document = parse(source);

  let count = document.decode_escapes(None).unwrap();

  assert_eq!(count, 0);
  assert_eq!(document.to_string(), source);
}

#[test]
fn test_decode_escapes_is_idempotent() {
  let mut document = parse("k: \"a \\U0001F971 b\"\n");

  document.decode_escapes(None).unwrap();
  let once = document.to_string();

  assert_eq!(document.decode_escapes(None).unwrap(), 0);
  assert_eq!(document.to_string(), once);
}

#[test]
fn test_decode_escapes_can_be_scoped_by_path() {
  let mut document = parse("a: \"\\U0001F971\"\nb: \"\\U0001F971\"\n");

  document.decode_escapes(Some("a")).unwrap();

  assert_eq!(document.to_string(), "a: \"🥱\"\nb: \"\\U0001F971\"\n");
}
