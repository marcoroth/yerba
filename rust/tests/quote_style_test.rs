use yerba::Document;

#[test]
fn test_enforce_quotes_to_double() {
  let yaml = "host: localhost\nname: 'myapp'\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    "host: \"localhost\"\nname: \"myapp\"\n"
  );
}

#[test]
fn test_enforce_quotes_to_single() {
  let yaml = "host: localhost\nname: \"myapp\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_quotes(&yerba::QuoteStyle::Single).unwrap();

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

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    "# Config\nhost: \"localhost\"\n# End\n"
  );
}

#[test]
fn test_enforce_quotes_noop_when_already_correct() {
  let yaml = "\"host\": \"localhost\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(document.to_string(), "\"host\": \"localhost\"\n");
}

#[test]
fn test_enforce_quotes_roundtrip() {
  let yaml = "host: localhost\nname: 'myapp'\ndesc: \"hello\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();
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

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(document.to_string(), "port: 5432\ncount: 10\nprice: 9.99\n");
}

#[test]
fn test_enforce_quotes_skips_booleans() {
  let yaml = "debug: true\nverbose: false\n";
  let mut document = Document::parse(yaml).unwrap();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

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

#[test]
fn test_enforce_key_style_to_double() {
  let yaml = "host: localhost\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Double, None)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "\"host\": localhost\n\"port\": 5432\n"
  );
}

#[test]
fn test_enforce_key_style_to_single() {
  let yaml = "host: localhost\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Single, None)
    .unwrap();

  assert_eq!(document.to_string(), "'host': localhost\n'port': 5432\n");
}

#[test]
fn test_enforce_key_style_scoped_to_path() {
  let yaml = "database:\n  host: localhost\n  port: 5432\napp:\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Double, Some("database"))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  \"host\": localhost\n  \"port\": 5432\napp:\n  name: myapp\n"
  );
}

#[test]
fn test_enforce_key_style_preserves_values() {
  let yaml = "host: \"localhost\"\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Double, None)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "\"host\": \"localhost\"\n\"port\": 5432\n"
  );
}

#[test]
fn test_enforce_key_style_nested() {
  let yaml = "\"database\":\n  \"host\": localhost\n  'port': 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Plain, None)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_enforce_both_key_and_value_style() {
  let yaml = "host: localhost\nname: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Double, None)
    .unwrap();
  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    "\"host\": \"localhost\"\n\"name\": \"myapp\"\n"
  );
}

#[test]
fn test_enforce_keys_double_values_single() {
  let yaml = "host: localhost\nname: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_key_style(&yerba::QuoteStyle::Double, None)
    .unwrap();
  document.enforce_quotes(&yerba::QuoteStyle::Single).unwrap();

  assert_eq!(
    document.to_string(),
    "\"host\": 'localhost'\n\"name\": 'myapp'\n"
  );
}

#[test]
fn test_enforce_quotes_scoped_to_single_key() {
  let yaml = "host: localhost\nport: 5432\nname: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes_at(&yerba::QuoteStyle::Single, Some("host"))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "host: 'localhost'\nport: 5432\nname: myapp\n"
  );
}

#[test]
fn test_enforce_quotes_scoped_to_nested_key() {
  let yaml = "database:\n  host: localhost\n  port: 5432\napp:\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes_at(&yerba::QuoteStyle::Double, Some("database"))
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: \"localhost\"\n  port: 5432\napp:\n  name: myapp\n"
  );
}
