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

  document.enforce_key_style(&yerba::KeyStyle::Plain, None).unwrap();

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

  document.enforce_key_style(&yerba::KeyStyle::Double, None).unwrap();

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

  document.enforce_key_style(&yerba::KeyStyle::Single, None).unwrap();

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

  document.enforce_key_style(&yerba::KeyStyle::Double, Some("database")).unwrap();

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

  document.enforce_key_style(&yerba::KeyStyle::Double, None).unwrap();

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

  document.enforce_key_style(&yerba::KeyStyle::Plain, None).unwrap();

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

  document.enforce_key_style(&yerba::KeyStyle::Double, None).unwrap();
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

  document.enforce_key_style(&yerba::KeyStyle::Double, None).unwrap();
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

  document.enforce_quotes_at(&yerba::QuoteStyle::Single, Some("host")).unwrap();

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

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("database")).unwrap();

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

#[test]
fn test_double_to_literal() {
  let mut document = parse(indoc! {r#"
    description: "Hello World"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        Hello World
    "}
  );
}

#[test]
fn test_single_to_literal() {
  let mut document = parse(indoc! {"
    description: 'Hello World'
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        Hello World
    "}
  );
}

#[test]
fn test_plain_to_literal() {
  let mut document = parse(indoc! {"
    description: Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        Hello World
    "}
  );
}

#[test]
fn test_double_to_folded() {
  let mut document = parse(indoc! {r#"
    description: "Hello World"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Folded, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: >-
        Hello World
    "}
  );
}

#[test]
fn test_literal_to_double() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      description: "Hello World"
    "#}
  );
}

#[test]
fn test_literal_multiline_to_double() {
  let mut document = parse(indoc! {"
    description: |-
      First line
      Second line
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      description: "First line\nSecond line"
    "#}
  );
}

#[test]
fn test_literal_single_line_to_single() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Single, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: 'Hello World'
    "}
  );
}

#[test]
fn test_literal_multiline_to_single_skipped() {
  let mut document = parse(indoc! {"
    description: |-
      First line
      Second line
  "});

  let original = document.to_string();

  document.enforce_quotes_at(&yerba::QuoteStyle::Single, Some("description")).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_literal_single_line_to_plain() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Plain, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: Hello World
    "}
  );
}

#[test]
fn test_literal_multiline_to_plain_skipped() {
  let mut document = parse(indoc! {"
    description: |-
      First line
      Second line
  "});

  let original = document.to_string();

  document.enforce_quotes_at(&yerba::QuoteStyle::Plain, Some("description")).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_literal_with_colon_to_plain_skipped() {
  let mut document = parse(indoc! {"
    description: |-
      key: value
  "});

  let original = document.to_string();

  document.enforce_quotes_at(&yerba::QuoteStyle::Plain, Some("description")).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_literal_to_folded() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Folded, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: >-
        Hello World
    "}
  );
}

#[test]
fn test_literal_to_literal_clip() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::LiteralClip, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |
        Hello World
    "}
  );
}

#[test]
fn test_literal_to_literal_keep() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::LiteralKeep, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |+
        Hello World
    "}
  );
}

#[test]
fn test_folded_to_literal() {
  let mut document = parse(indoc! {"
    description: >-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        Hello World
    "}
  );
}

#[test]
fn test_folded_to_folded_keep() {
  let mut document = parse(indoc! {"
    description: >-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::FoldedKeep, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: >+
        Hello World
    "}
  );
}

#[test]
fn test_literal_noop_when_already_correct() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  let original = document.to_string();

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_literal_preserves_other_fields() {
  let mut document = parse(indoc! {r#"
    title: "My Title"
    description: "Hello World"
    name: "test"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      title: "My Title"
      description: |-
        Hello World
      name: "test"
    "#}
  );
}

#[test]
fn test_literal_in_sequence() {
  let mut document = parse(indoc! {r#"
    - id: talk-1
      description: "First talk"
    - id: talk-2
      description: "Second talk"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("[].description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      - id: talk-1
        description: |-
          First talk
      - id: talk-2
        description: |-
          Second talk
    "}
  );
}

#[test]
fn test_literal_skips_empty_string() {
  let mut document = parse(indoc! {r#"
    description: ""
  "#});

  let original = document.to_string();

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_literal_multiline_to_single_returns_warning() {
  let mut document = parse(indoc! {"
    description: |-
      First line
      Second line
  "});

  let warnings = document.enforce_quotes_at(&yerba::QuoteStyle::Single, Some("description")).unwrap();

  assert_eq!(warnings.len(), 1);
  assert!(warnings[0].contains("skipped"));
  assert!(warnings[0].contains("single"));
}

#[test]
fn test_literal_multiline_to_plain_returns_warning() {
  let mut document = parse(indoc! {"
    description: |-
      First line
      Second line
  "});

  let warnings = document.enforce_quotes_at(&yerba::QuoteStyle::Plain, Some("description")).unwrap();

  assert_eq!(warnings.len(), 1);
  assert!(warnings[0].contains("skipped"));
  assert!(warnings[0].contains("plain"));
}

#[test]
fn test_literal_with_colon_to_plain_returns_warning() {
  let mut document = parse(indoc! {"
    description: |-
      key: value
  "});

  let warnings = document.enforce_quotes_at(&yerba::QuoteStyle::Plain, Some("description")).unwrap();

  assert_eq!(warnings.len(), 1);
  assert!(warnings[0].contains("skipped"));
}

#[test]
fn test_successful_conversion_returns_no_warnings() {
  let mut document = parse(indoc! {r#"
    description: "Hello World"
  "#});

  let warnings = document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert!(warnings.is_empty());
}

#[test]
fn test_mixed_document_to_literal_scoped() {
  let mut document = parse(indoc! {r#"
    - id: "talk-1"
      title: "First Talk"
      description: "A great talk about Ruby"
      speakers:
        - "Alice"
    - id: "talk-2"
      title: "Second Talk"
      description: "Another great talk"
      speakers:
        - "Bob"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("[].description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: "talk-1"
        title: "First Talk"
        description: |-
          A great talk about Ruby
        speakers:
          - "Alice"
      - id: "talk-2"
        title: "Second Talk"
        description: |-
          Another great talk
        speakers:
          - "Bob"
    "#}
  );
}

#[test]
fn test_mixed_document_literal_to_double_scoped() {
  let mut document = parse(indoc! {r#"
    - id: "talk-1"
      title: "First Talk"
      description: |-
        A great talk about Ruby
      speakers:
        - "Alice"
    - id: "talk-2"
      title: "Second Talk"
      description: |-
        Another great talk
      speakers:
        - "Bob"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("[].description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: "talk-1"
        title: "First Talk"
        description: "A great talk about Ruby"
        speakers:
          - "Alice"
      - id: "talk-2"
        title: "Second Talk"
        description: "Another great talk"
        speakers:
          - "Bob"
    "#}
  );
}

#[test]
fn test_mixed_document_all_values_to_double_preserves_block_scalars() {
  let mut document = parse(indoc! {"
    host: localhost
    port: 5432
    name: 'myapp'
    description: |-
      A simple config
    tags:
      - ruby
      - rust
  "});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      host: "localhost"
      port: 5432
      name: "myapp"
      description: |-
        A simple config
      tags:
        - "ruby"
        - "rust"
    "#}
  );
}

#[test]
fn test_mixed_document_multiline_block_skipped_to_single() {
  let mut document = parse(indoc! {r#"
    title: "My Talk"
    description: |-
      First paragraph.
      Second paragraph.
    speaker: "Alice"
  "#});

  let original = document.to_string();

  let warnings = document.enforce_quotes_at(&yerba::QuoteStyle::Single, Some("[].description")).unwrap();

  assert_eq!(document.to_string(), original);
  assert!(warnings.is_empty());
}

#[test]
fn test_mixed_document_multiline_block_skipped_in_sequence() {
  let mut document = parse(indoc! {r#"
    - id: "talk-1"
      title: "First"
      description: |-
        Line one
        Line two
    - id: "talk-2"
      title: "Second"
      description: "Short"
  "#});

  let warnings = document.enforce_quotes_at(&yerba::QuoteStyle::Single, Some("[].description")).unwrap();

  assert_eq!(warnings.len(), 1);
  assert!(warnings[0].contains("skipped"));

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      - id: "talk-1"
        title: "First"
        description: |-
          Line one
          Line two
      - id: "talk-2"
        title: "Second"
        description: 'Short'
    "#}
  );
}

#[test]
fn test_literal_strip_to_clip_to_keep_roundtrip() {
  let mut document = parse(indoc! {"
    description: |-
      Hello World
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::LiteralClip, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |
        Hello World
    "}
  );

  document.enforce_quotes_at(&yerba::QuoteStyle::LiteralKeep, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |+
        Hello World
    "}
  );

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        Hello World
    "}
  );
}

#[test]
fn test_double_to_literal_and_back() {
  let mut document = parse(indoc! {r#"
    description: "Hello World"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        Hello World
    "}
  );

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      description: "Hello World"
    "#}
  );
}

#[test]
fn test_double_noop_does_not_corrupt_line_continuation() {
  let mut document = parse(indoc! {r#"
    maps:
      google: "https://maps.app.goo.gl/BkW4eSnUjqwk8tV8A"
      apple:
        "https://example.com/place?address=17%20Jubilee%20St&map=\
        transit"
  "#});

  let original = document.to_string();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_double_noop_preserves_multiline_double_quoted() {
  let mut document = parse(indoc! {r#"
    url:
      "https://example.com/very-long-url\
      /continuation"
  "#});

  let original = document.to_string();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_double_noop_with_backslash_in_value() {
  let mut document = parse(indoc! {r#"
    path: "C:\\Users\\test"
  "#});

  let original = document.to_string();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_single_noop_preserves_value() {
  let mut document = parse(indoc! {"
    name: 'myapp'
  "});

  let original = document.to_string();

  document.enforce_quotes(&yerba::QuoteStyle::Single).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_plain_noop_preserves_value() {
  let mut document = parse(indoc! {"
    name: myapp
  "});

  let original = document.to_string();

  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_double_on_mixed_file_does_not_touch_already_double() {
  let mut document = parse(indoc! {r#"
    name: "Brighton Dome"
    address:
      street: "Church St"
      city: "Brighton"
    maps:
      google: "https://maps.app.goo.gl/BkW4eSnUjqwk8tV8A"
      apple:
        "https://example.com/place?map=\
        transit"
  "#});

  let original = document.to_string();

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(document.to_string(), original);
}

#[test]
fn test_double_converts_plain_but_preserves_existing_double() {
  let mut document = parse(indoc! {r#"
    host: localhost
    url:
      "https://example.com/path?key=\
      value"
  "#});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      host: "localhost"
      url:
        "https://example.com/path?key=\
        value"
    "#}
  );
}

#[test]
fn test_double_with_escaped_newlines_to_literal() {
  let mut document = parse(indoc! {r#"
    description: "First line.\nSecond line.\nThird line."
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        First line.
        Second line.
        Third line.
    "}
  );
}

#[test]
fn test_double_with_escaped_tab_to_literal() {
  let mut document = parse(indoc! {r#"
    description: "Hello\tWorld"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(document.to_string(), "description: |-\n  Hello\tWorld\n");
}

#[test]
fn test_literal_with_newlines_to_double() {
  let mut document = parse(indoc! {"
    description: |-
      First line.
      Second line.
      Third line.
  "});

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      description: "First line.\nSecond line.\nThird line."
    "#}
  );
}

#[test]
fn test_double_with_newlines_to_literal_and_back() {
  let mut document = parse(indoc! {r#"
    description: "First.\nSecond."
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      description: |-
        First.
        Second.
    "}
  );

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      description: "First.\nSecond."
    "#}
  );
}

#[test]
fn test_double_with_escaped_backslash_preserved() {
  let mut document = parse(indoc! {r#"
    path: "C:\\Users\\test"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("path")).unwrap();

  assert_eq!(document.to_string(), "path: |-\n  C:\\Users\\test\n");
}

#[test]
fn test_literal_with_backslash_to_double() {
  let mut document = parse("path: |-\n  C:\\Users\\test\n");

  document.enforce_quotes_at(&yerba::QuoteStyle::Double, Some("path")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      path: "C:\\Users\\test"
    "#}
  );
}

#[test]
fn test_double_to_literal_nested() {
  let mut document = parse(indoc! {r#"
    database:
      description: "A database config"
  "#});

  document.enforce_quotes_at(&yerba::QuoteStyle::Literal, Some("database.description")).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {"
      database:
        description: |-
          A database config
    "}
  );
}
