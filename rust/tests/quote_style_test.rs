mod support;
use indoc::indoc;
use support::parse;

#[test]
fn test_enforce_quotes_to_double() {
  let mut document = parse(indoc! {"
    host: localhost
    name: 'myapp'
  "});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      host: "localhost"
      name: "myapp"
    "#}
  );
}

#[test]
fn test_enforce_quotes_to_single() {
  let mut document = parse(indoc! {r#"
    host: localhost
    name: "myapp"
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Single).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: 'localhost'
      name: 'myapp'
    "}
  );
}

#[test]
fn test_enforce_quotes_to_plain() {
  let mut document = parse(indoc! {r#"
    host: "localhost"
    name: 'myapp'
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      name: myapp
    "}
  );
}

#[test]
fn test_enforce_quotes_preserves_comments() {
  let mut document = parse(indoc! {"
    # Config
    host: localhost
    # End
  "});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      # Config
      host: "localhost"
      # End
    "#}
  );
}

#[test]
fn test_enforce_quotes_noop_when_already_correct() {
  let mut document = parse(indoc! {r#"
    "host": "localhost"
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      "host": "localhost"
    "#}
  );
}

#[test]
fn test_enforce_quotes_roundtrip() {
  let mut document = parse(indoc! {r#"
    host: localhost
    name: 'myapp'
    desc: "hello"
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();
  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      name: myapp
      desc: hello
    "}
  );
}

#[test]
fn test_enforce_quotes_skips_numbers() {
  let mut document = parse(indoc! {"
    port: 5432
    count: 10
    price: 9.99
  "});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      port: 5432
      count: 10
      price: 9.99
    "}
  );
}

#[test]
fn test_enforce_quotes_skips_booleans() {
  let mut document = parse(indoc! {"
    debug: true
    verbose: false
  "});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      debug: true
      verbose: false
    "}
  );
}

#[test]
fn test_enforce_key_style() {
  let mut document = parse(indoc! {r#"
    "host": localhost
    'port': 5432
  "#});

  document.enforce_key_style(&yerba::QuoteStyle::Plain, None).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: localhost
      port: 5432
    "}
  );
}

#[test]
fn test_enforce_key_style_to_double() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  document.enforce_key_style(&yerba::QuoteStyle::Double, None).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      "host": localhost
      "port": 5432
    "#}
  );
}

#[test]
fn test_enforce_key_style_to_single() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
  "});

  document.enforce_key_style(&yerba::QuoteStyle::Single, None).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      'host': localhost
      'port': 5432
    "}
  );
}

#[test]
fn test_enforce_key_style_scoped_to_path() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
    app:
      name: myapp
  "});

  document
    .enforce_key_style(&yerba::QuoteStyle::Double, Some("database"))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      database:
        "host": localhost
        "port": 5432
      app:
        name: myapp
    "#}
  );
}

#[test]
fn test_enforce_key_style_preserves_values() {
  let mut document = parse(indoc! {r#"
    host: "localhost"
    port: 5432
  "#});

  document.enforce_key_style(&yerba::QuoteStyle::Double, None).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      "host": "localhost"
      "port": 5432
    "#}
  );
}

#[test]
fn test_enforce_key_style_nested() {
  let mut document = parse(indoc! {r#"
    "database":
      "host": localhost
      'port': 5432
  "#});

  document.enforce_key_style(&yerba::QuoteStyle::Plain, None).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        host: localhost
        port: 5432
    "}
  );
}

#[test]
fn test_enforce_both_key_and_value_style() {
  let mut document = parse(indoc! {"
    host: localhost
    name: myapp
  "});

  document.enforce_key_style(&yerba::QuoteStyle::Double, None).unwrap();
  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      "host": "localhost"
      "name": "myapp"
    "#}
  );
}

#[test]
fn test_enforce_keys_double_values_single() {
  let mut document = parse(indoc! {"
    host: localhost
    name: myapp
  "});

  document.enforce_key_style(&yerba::QuoteStyle::Double, None).unwrap();
  document.enforce_quotes(&yerba::QuoteStyle::Single).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      "host": 'localhost'
      "name": 'myapp'
    "#}
  );
}

#[test]
fn test_enforce_quotes_scoped_to_single_key() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    name: myapp
  "});

  document
    .enforce_quotes_at(&yerba::QuoteStyle::Single, Some("host"))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      host: 'localhost'
      port: 5432
      name: myapp
    "}
  );
}

#[test]
fn test_enforce_quotes_scoped_to_nested_key() {
  let mut document = parse(indoc! {"
    database:
      host: localhost
      port: 5432
    app:
      name: myapp
  "});

  document
    .enforce_quotes_at(&yerba::QuoteStyle::Double, Some("database"))
    .unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      database:
        host: "localhost"
        port: 5432
      app:
        name: myapp
    "#}
  );
}

#[test]
fn test_enforce_quotes_skips_value_with_inner_double_quotes() {
  let mut document = parse(indoc! {r#"
    title: 'Panel "Post-Rails world" - wroc_love.rb'
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      title: 'Panel "Post-Rails world" - wroc_love.rb'
    "#}
  );
}

#[test]
fn test_enforce_quotes_single_escapes_inner_single_quotes() {
  let mut document = parse(indoc! {r#"
    title: "it's a test"
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Single).unwrap();

  assert_eq!(document.to_string(), "title: 'it''s a test'\n");
}

#[test]
fn test_enforce_quotes_plain_skips_value_with_special_chars() {
  let mut document = parse(indoc! {r#"
    title: "hello: world"
    subtitle: "no # comment"
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      title: "hello: world"
      subtitle: "no # comment"
    "#}
  );
}

#[test]
fn test_enforce_quotes_plain_strips_safe_values() {
  let mut document = parse(indoc! {r#"
    title: "hello world"
    name: "myapp"
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      title: hello world
      name: myapp
    "}
  );
}

#[test]
fn test_enforce_quotes_double_with_plain_containing_quotes() {
  let mut document = parse("name: simple\n");

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(document.to_string(), "name: \"simple\"\n");
}
