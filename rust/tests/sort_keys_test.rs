use yerba::Document;

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

  document.sort_keys("database", &["name", "pool"]).unwrap();

  assert_eq!(
    document.to_string(),
    "database:\n  name: myapp\n  pool: 10\n  host: localhost\n  port: 5432\n"
  );
}

#[test]
fn test_sort_keys_with_bracket_path() {
  let yaml = "- name: first\n  id: 1\n- name: second\n  id: 2\n";
  let mut document = Document::parse(yaml).unwrap();

  document.sort_keys("[]", &["id", "name"]).unwrap();

  assert_eq!(
    document.to_string(),
    "- id: 1\n  name: first\n- id: 2\n  name: second\n"
  );
}

#[test]
fn test_validate_sort_keys_with_bracket_path() {
  let yaml = "- name: first\n  id: 1\n";
  let document = Document::parse(yaml).unwrap();

  assert!(document.validate_sort_keys("[]", &["id", "name"]).is_ok());
  assert!(document.validate_sort_keys("[]", &["id"]).is_err());
}
