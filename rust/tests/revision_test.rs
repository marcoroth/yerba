mod support;
use support::parse;

#[test]
fn test_revision_starts_at_zero() {
  let document = parse("host: localhost");

  assert_eq!(document.revision(), 0);
}

#[test]
fn test_revision_is_unchanged_by_reads() {
  let document = parse("host: localhost\nport: 5432");

  document.get("host").unwrap();
  document.source("host").unwrap();
  document.to_string();

  assert_eq!(document.revision(), 0);
}

#[test]
fn test_revision_changes_on_set() {
  let mut document = parse("host: localhost");

  document.set("host", "0.0.0.0").unwrap();

  let after_set = document.revision();

  assert!(after_set > 0);

  document.set("host", "127.0.0.1").unwrap();

  assert!(document.revision() > after_set);
}

#[test]
fn test_revision_changes_on_delete() {
  let mut document = parse("host: localhost\nport: 5432");
  let before = document.revision();

  document.delete("port").unwrap();

  assert!(document.revision() > before);
}

#[test]
fn test_revision_changes_on_append() {
  let mut document = parse("items:\n  - first");
  let before = document.revision();

  document.append("items", "second").unwrap();

  assert!(document.revision() > before);
}

#[test]
fn test_revision_changes_on_rename() {
  let mut document = parse("host: localhost");
  let before = document.revision();

  document.rename("host", "hostname").unwrap();

  assert!(document.revision() > before);
}

#[test]
fn test_revision_changes_on_sort_keys() {
  let mut document = parse("port: 5432\nhost: localhost");
  let before = document.revision();

  document.sort_keys("", &["host", "port"]).unwrap();

  assert!(document.revision() > before);
}

#[test]
fn test_revision_changes_on_quote_style_change() {
  let mut document = parse("name: Alice");
  let before = document.revision();

  document.set_scalar_style("name", &yerba::QuoteStyle::Double).unwrap();

  assert!(document.revision() > before);
}

#[test]
fn test_revision_is_unchanged_when_an_edit_fails() {
  let mut document = parse("host: localhost");

  assert!(document.set("missing", "value").is_err());
  assert_eq!(document.revision(), 0);
}
