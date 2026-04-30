use yerba::Document;

#[test]
fn test_get_plain_scalar() {
  let document = Document::parse("host: localhost").unwrap();

  assert_eq!(document.get("host"), Some("localhost".to_string()));
}

#[test]
fn test_get_double_quoted_scalar() {
  let document = Document::parse("name: \"myapp\"").unwrap();

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_single_quoted_scalar() {
  let document = Document::parse("name: 'myapp'").unwrap();

  assert_eq!(document.get("name"), Some("myapp".to_string()));
}

#[test]
fn test_get_nested_path() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get("database.host"), Some("localhost".to_string()));
  assert_eq!(document.get("database.port"), Some("5432".to_string()));
}

#[test]
fn test_get_deeply_nested() {
  let yaml = "a:\n  b:\n    c: deep\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.get("a.b.c"), Some("deep".to_string()));
}

#[test]
fn test_get_nonexistent_path() {
  let document = Document::parse("host: localhost").unwrap();

  assert_eq!(document.get("missing"), None);
}

#[test]
fn test_set_plain_scalar() {
  let mut document = Document::parse("host: localhost").unwrap();

  document.set("host", "0.0.0.0").unwrap();

  assert_eq!(document.to_string(), "host: 0.0.0.0");
}

#[test]
fn test_set_preserves_double_quotes() {
  let mut document = Document::parse("name: \"myapp\"").unwrap();

  document.set("name", "newapp").unwrap();

  assert_eq!(document.to_string(), "name: \"newapp\"");
}

#[test]
fn test_set_preserves_single_quotes() {
  let mut document = Document::parse("name: 'myapp'").unwrap();

  document.set("name", "newapp").unwrap();

  assert_eq!(document.to_string(), "name: 'newapp'");
}

#[test]
fn test_set_nested_path() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("database.host", "0.0.0.0").unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: 0.0.0.0\n  port: 5432\n"
  );
}

#[test]
fn test_set_preserves_comments() {
  let yaml = "# Database config\nhost: localhost\n# Port\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.set("host", "0.0.0.0").unwrap();

  assert_eq!(
    document.to_string(),
    "# Database config\nhost: 0.0.0.0\n# Port\nport: 5432\n"
  );
}

#[test]
fn test_roundtrip_no_changes() {
  let yaml = "# comment\nkey: value\nnested:\n  a: 1\n  b: 'two'\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.to_string(), yaml);
}

#[test]
fn test_append_to_sequence() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document.append("tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - ruby\n  - rust\n  - yaml\n"
  );
}

#[test]
fn test_append_to_nested_sequence() {
  let yaml = "app:\n  tags:\n    - ruby\n    - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document.append("app.tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    "app:\n  tags:\n    - ruby\n    - rust\n    - yaml\n"
  );
}

#[test]
fn test_append_preserves_comments() {
  let yaml = "# Tags\ntags:\n  - ruby\n  - rust\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.append("tags", "yaml").unwrap();

  assert_eq!(
    document.to_string(),
    "# Tags\ntags:\n  - ruby\n  - rust\n  - yaml\n# End\n"
  );
}

#[test]
fn test_delete_key() {
  let yaml = "host: localhost\nport: 5432\nname: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.delete("port").unwrap();

  assert_eq!(document.to_string(), "host: localhost\nname: myapp\n");
}

#[test]
fn test_delete_nested_key() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.delete("database.port").unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  host: localhost\n  name: myapp\n"
  );
}

#[test]
fn test_delete_preserves_comments() {
  let yaml = "# Config\nhost: localhost\nport: 5432\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.delete("port").unwrap();

  assert_eq!(document.to_string(), "# Config\nhost: localhost\n# End\n");
}

#[test]
fn test_delete_nonexistent_key() {
  let mut document = Document::parse("host: localhost\n").unwrap();

  assert!(document.delete("missing").is_err());
}

#[test]
fn test_remove_from_sequence() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.remove("tags", "rust").unwrap();

  assert_eq!(document.to_string(), "tags:\n  - ruby\n  - yaml\n");
}

#[test]
fn test_remove_from_nested_sequence() {
  let yaml = "app:\n  tags:\n    - ruby\n    - rust\n    - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.remove("app.tags", "rust").unwrap();

  assert_eq!(
    document.to_string(),
    "app:\n  tags:\n    - ruby\n    - yaml\n"
  );
}

#[test]
fn test_remove_nonexistent_item() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document.remove("tags", "missing").is_err());
}

#[test]
fn test_remove_from_non_sequence() {
  let mut document = Document::parse("host: localhost\n").unwrap();

  assert!(document.remove("host", "value").is_err());
}

#[test]
fn test_rename_key() {
  let yaml = "host: localhost\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("host", "hostname").unwrap();

  assert_eq!(document.to_string(), "hostname: localhost\nport: 5432\n");
}

#[test]
fn test_rename_nested_key() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("database.host", "hostname").unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  hostname: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_rename_preserves_comments() {
  let yaml = "# Config\nhost: localhost\n# Port\nport: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("host", "hostname").unwrap();

  assert_eq!(
    document.to_string(),
    "# Config\nhostname: localhost\n# Port\nport: 5432\n"
  );
}

#[test]
fn test_rename_preserves_quoted_key() {
  let yaml = "'host': localhost\n";
  let mut document = Document::parse(yaml).unwrap();

  document.rename("host", "hostname").unwrap();

  assert_eq!(document.to_string(), "'hostname': localhost\n");
}

#[test]
fn test_rename_nonexistent_key() {
  let mut document = Document::parse("host: localhost\n").unwrap();

  assert!(document.rename("missing", "new_name").is_err());
}

#[test]
fn test_move_item_by_index() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 2, 0).unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - yaml\n  - ruby\n  - rust\n"
  );
}

#[test]
fn test_move_item_forward() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 0, 2).unwrap();

  assert_eq!(
    document.to_string(),
    "tags:\n  - rust\n  - yaml\n  - ruby\n"
  );
}

#[test]
fn test_move_item_same_index() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 0, 0).unwrap();

  assert_eq!(document.to_string(), "tags:\n  - ruby\n  - rust\n");
}

#[test]
fn test_move_item_out_of_bounds() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document.move_item("tags", 5, 0).is_err());
}

#[test]
fn test_move_item_preserves_comments() {
  let yaml = "# Tags\ntags:\n  - ruby\n  - rust\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_item("tags", 1, 0).unwrap();

  assert_eq!(
    document.to_string(),
    "# Tags\ntags:\n  - rust\n  - ruby\n# End\n"
  );
}

#[test]
fn test_resolve_sequence_index_by_name() {
  let yaml = "tags:\n  - ruby\n  - rust\n  - yaml\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_sequence_index("tags", "ruby").unwrap(), 0);
  assert_eq!(document.resolve_sequence_index("tags", "rust").unwrap(), 1);
  assert_eq!(document.resolve_sequence_index("tags", "yaml").unwrap(), 2);
}

#[test]
fn test_resolve_sequence_index_by_number() {
  let yaml = "tags:\n  - ruby\n  - rust\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_sequence_index("tags", "0").unwrap(), 0);
  assert_eq!(document.resolve_sequence_index("tags", "1").unwrap(), 1);
}

#[test]
fn test_move_key_by_name() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("database", 2, 0).unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  name: myapp\n  host: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_move_key_forward() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("database", 0, 2).unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  port: 5432\n  name: myapp\n  host: localhost\n"
  );
}

#[test]
fn test_move_key_preserves_comments() {
  let yaml = "# Config\ndatabase:\n  host: localhost\n  port: 5432\n# End\n";
  let mut document = Document::parse(yaml).unwrap();

  document.move_key("database", 1, 0).unwrap();

  assert_eq!(
    document.to_string(),
    "# Config\ndatabase:\n  port: 5432\n  host: localhost\n# End\n"
  );
}

#[test]
fn test_move_key_out_of_bounds() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let mut document = Document::parse(yaml).unwrap();

  assert!(document.move_key("database", 5, 0).is_err());
}

#[test]
fn test_resolve_key_index_by_name() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_key_index("database", "host").unwrap(), 0);
  assert_eq!(document.resolve_key_index("database", "port").unwrap(), 1);
  assert_eq!(document.resolve_key_index("database", "name").unwrap(), 2);
}

#[test]
fn test_resolve_key_index_by_number() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n";
  let document = Document::parse(yaml).unwrap();

  assert_eq!(document.resolve_key_index("database", "0").unwrap(), 0);
  assert_eq!(document.resolve_key_index("database", "1").unwrap(), 1);
}

#[test]
fn test_sort_keys() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n  pool: 10\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .sort_keys("database", &["name", "host", "port", "pool"])
    .unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  name: myapp\n  host: localhost\n  port: 5432\n  pool: 10\n"
  );
}

#[test]
fn test_sort_keys_partial_order() {
  let yaml = "database:\n  host: localhost\n  port: 5432\n  name: myapp\n  pool: 10\n";
  let mut document = Document::parse(yaml).unwrap();

  // Only specify some keys — unspecified ones keep original relative order at the end
  document.sort_keys("database", &["name", "pool"]).unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  name: myapp\n  pool: 10\n  host: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_enforce_quotes_to_double() {
  let yaml = "host: localhost\nname: 'myapp'\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "\"host\": \"localhost\"\n\"name\": \"myapp\"\n"
  );
}

#[test]
fn test_enforce_quotes_to_single() {
  let yaml = "host: localhost\nname: \"myapp\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::SingleQuoted)
    .unwrap();

  assert_eq!(
    document.to_string(),
    "'host': 'localhost'\n'name': 'myapp'\n"
  );
}

#[test]
fn test_enforce_quotes_to_plain() {
  let yaml = "\"host\": \"localhost\"\n'name': 'myapp'\n";
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
    "# Config\n\"host\": \"localhost\"\n# End\n"
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
  let yaml = "host: localhost\nname: 'myapp'\nport: \"5432\"\n";
  let mut document = Document::parse(yaml).unwrap();

  document
    .enforce_quotes(&yerba::QuoteStyle::DoubleQuoted)
    .unwrap();
  document.enforce_quotes(&yerba::QuoteStyle::Plain).unwrap();

  assert_eq!(
    document.to_string(),
    "host: localhost\nname: myapp\nport: 5432\n"
  );
}
