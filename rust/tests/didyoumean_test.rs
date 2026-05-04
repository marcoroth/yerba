use yerba::didyoumean::{didyoumean, didyoumean_ranked};

#[test]
fn test_didyoumean_closest_match() {
  let list = vec!["database".to_string(), "tags".to_string(), "host".to_string()];
  assert_eq!(didyoumean("databse", &list), Some("database".to_string()));
}

#[test]
fn test_didyoumean_empty_list() {
  assert_eq!(didyoumean("hello", &[]), None);
}

#[test]
fn test_didyoumean_case_insensitive() {
  let list = vec!["Database".to_string()];
  assert_eq!(didyoumean("database", &list), Some("Database".to_string()));
}

#[test]
fn test_didyoumean_exact_match() {
  let list = vec!["host".to_string(), "port".to_string()];
  assert_eq!(didyoumean("host", &list), Some("host".to_string()));
}

#[test]
fn test_didyoumean_sorts_alphabetically_on_tie() {
  let list = vec!["bbb".to_string(), "aaa".to_string()];
  assert_eq!(didyoumean("ccc", &list), Some("aaa".to_string()));
}

#[test]
fn test_didyoumean_ranked_with_threshold() {
  let list = vec!["database.host".to_string(), "database.port".to_string(), "tags".to_string()];

  let results = didyoumean_ranked("databse.host", &list, 3);
  assert_eq!(results, vec!["database.host", "database.port"]);
}

#[test]
fn test_didyoumean_ranked_empty_on_high_distance() {
  let list = vec!["abc".to_string()];
  let results = didyoumean_ranked("xyz", &list, 1);
  assert!(results.is_empty());
}

#[test]
fn test_didyoumean_ranked_respects_order() {
  let list = vec![
    "tags".to_string(),
    "database".to_string(),
    "database.host".to_string(),
    "database.port".to_string(),
  ];

  let results = didyoumean_ranked("database", &list, 5);
  assert_eq!(results[0], "database");
  assert!(results.len() > 1);
}
