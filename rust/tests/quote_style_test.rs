use yerba::Document;

#[test]
fn test_enforce_quotes_to_double() {
  let yaml = "host: localhost\nname: 'myapp'\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "host: \"localhost\"\nname: \"myapp\"\n"
  );
}

#[test]
fn test_enforce_quotes_to_single() {
  let yaml = "host: localhost\nname: \"myapp\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::SingleQuoted)
    .unwrap();

  assert_eq!(document.to_string(), "host: 'localhost'\nname: 'myapp'\n");
}

#[test]
fn test_enforce_quotes_to_plain() {
  let yaml = "host: \"localhost\"\nname: 'myapp'\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(document.to_string(), "host: localhost\nname: myapp\n");
}

#[test]
fn test_enforce_quotes_preserves_comments() {
  let yaml = "# Config\nhost: localhost\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "# Config\nhost: \"localhost\"\n# End\n"
  );
}

#[test]
fn test_enforce_quotes_noop_when_already_correct() {
  let yaml = "\"host\": \"localhost\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();

  assert_eq!(document.to_string(), "\"host\": \"localhost\"\n");
}

#[test]
fn test_enforce_quotes_roundtrip() {
  let yaml = "host: localhost\nname: 'myapp'\ndesc: \"hello\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();
  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(
    document.to_string(),
    "host: localhost\nname: myapp\ndesc: hello\n"
  );
}

#[test]
fn test_enforce_quotes_skips_numbers() {
  let yaml = "port: 5432\ncount: 10\nprice: 9.99\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();

  assert_eq!(document.to_string(), "port: 5432\ncount: 10\nprice: 9.99\n");
}

#[test]
fn test_enforce_quotes_skips_booleans() {
  let yaml = "debug: true\nverbose: false\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();

  assert_eq!(document.to_string(), "debug: true\nverbose: false\n");
}

#[test]
fn test_enforce_key_style() {
  let yaml = "\"host\": localhost\n'port': 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Plain, None)
    .unwrap();

  assert_eq!(document.to_string(), "host: localhost\nport: 5432\n");
}
