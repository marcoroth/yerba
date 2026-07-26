mod support;
use indoc::indoc;
use support::parse;
use yerba::InsertPosition;

#[test]
fn test_index_into_a_flow_sequence() {
  let document = parse("tags: [ruby, rails]\n");

  assert_eq!(document.get("tags[0]"), Some("ruby".to_string()));
  assert_eq!(document.get("tags[1]"), Some("rails".to_string()));
  assert_eq!(document.get("tags[2]"), None);
}

#[test]
fn test_all_items_of_a_flow_sequence() {
  let document = parse("tags: [ruby, rails, yaml]\n");

  assert_eq!(document.get_all("tags[]"), vec!["ruby", "rails", "yaml"]);
}

#[test]
fn test_key_of_a_flow_map() {
  let document = parse("venue: {city: Berlin, country: DE}\n");

  assert_eq!(document.get("venue.city"), Some("Berlin".to_string()));
  assert_eq!(document.get("venue.country"), Some("DE".to_string()));
  assert_eq!(document.get("venue.missing"), None);
}

#[test]
fn test_all_keys_of_a_flow_map() {
  let document = parse("venue: {city: Berlin, country: DE}\n");

  assert_eq!(document.get_all("venue.*"), vec!["Berlin", "DE"]);
}

#[test]
fn test_flow_map_inside_a_flow_sequence() {
  let document = parse("items: [{a: 1}, {b: 2}]\n");

  assert_eq!(document.get("items[0].a"), Some("1".to_string()));
  assert_eq!(document.get("items[1].b"), Some("2".to_string()));
  assert_eq!(document.get_all("items[].*"), vec!["1", "2"]);
}

#[test]
fn test_flow_sequence_inside_a_block_sequence() {
  let document = parse(indoc! {"
    - tags: [a, b]
    - tags: [c]
  "});

  assert_eq!(document.get_all("[].tags[]"), vec!["a", "b", "c"]);
  assert_eq!(document.get("[1].tags[0]"), Some("c".to_string()));
}

#[test]
fn test_flow_nodes_resolve_to_concrete_selectors() {
  let document = parse("tags: [ruby, rails]\nvenue: {city: Berlin, country: DE}\n");

  assert_eq!(document.resolve_selectors("tags[]"), vec!["tags[0]", "tags[1]"]);
  assert_eq!(document.resolve_selectors("venue.*"), vec!["venue.city", "venue.country"]);
}

#[test]
fn test_a_key_lookup_does_not_reach_into_a_flow_sequences_items() {
  let document = parse("items: [{a: 1}]\n");

  assert_eq!(document.get("items.a"), None);
}

#[test]
fn test_exists_for_flow_paths() {
  let document = parse("tags: [ruby]\nvenue: {city: Berlin}\n");

  assert!(document.exists("tags[0]"));
  assert!(document.exists("venue.city"));
  assert!(!document.exists("venue.missing"));
  assert!(!document.exists("tags[9]"));
}

#[test]
fn test_set_a_flow_sequence_entry() {
  let mut document = parse("tags: [ruby, rails]\nname: x\n");

  document.set("tags[0]", "crystal").unwrap();

  assert_eq!(document.to_string(), "tags: [crystal, rails]\nname: x\n");
}

#[test]
fn test_set_a_flow_map_value() {
  let mut document = parse("venue: {city: Berlin, country: DE}\n");

  document.set("venue.city", "Hamburg").unwrap();

  assert_eq!(document.to_string(), "venue: {city: Hamburg, country: DE}\n");
}

#[test]
fn test_set_keeps_each_flow_entrys_quote_style() {
  let mut document = parse("tags: [\"ruby\", 'rails']\n");

  document.set("tags[0]", "crystal").unwrap();
  document.set("tags[1]", "erlang").unwrap();

  assert_eq!(document.to_string(), "tags: [\"crystal\", 'erlang']\n");
}

#[test]
fn test_delete_refuses_a_flow_sequence_entry() {
  let mut document = parse("tags: [ruby, rails]\n");

  let error = document.delete("tags[0]").unwrap_err();

  assert!(error.to_string().contains("flow collections are not writable"), "{}", error);
  assert_eq!(document.to_string(), "tags: [ruby, rails]\n");
}

#[test]
fn test_delete_refuses_a_flow_map_entry() {
  let mut document = parse("venue: {city: Berlin, country: DE}\n");

  assert!(document.delete("venue.city").is_err());
  assert_eq!(document.to_string(), "venue: {city: Berlin, country: DE}\n");
}

#[test]
fn test_remove_refuses_a_flow_sequence() {
  let mut document = parse("tags: [ruby, rails]\n");

  assert!(document.remove("tags", "ruby").is_err());
  assert_eq!(document.to_string(), "tags: [ruby, rails]\n");
}

#[test]
fn test_insert_refuses_an_occupied_flow_map() {
  let mut document = parse("venue: {city: Berlin}\n");

  assert!(document.insert_into("venue.zip", "12345", InsertPosition::Last).is_err());
  assert_eq!(document.to_string(), "venue: {city: Berlin}\n");
}

#[test]
fn test_insert_still_replaces_an_empty_flow_map() {
  let mut document = parse("metadata: {}\n");

  document.insert_into("metadata.source", "youtube", InsertPosition::Last).unwrap();

  assert_eq!(document.to_string(), "metadata:\n  source: youtube\n");
}

#[test]
fn test_deleting_the_whole_entry_holding_a_flow_collection_still_works() {
  let mut document = parse("tags: [ruby]\nvenue: {city: Berlin}\nname: x\n");

  document.delete("tags").unwrap();
  document.delete("venue").unwrap();

  assert_eq!(document.to_string(), "name: x\n");
}

#[test]
fn test_block_style_is_unaffected() {
  let mut document = parse(indoc! {"
    tags:
      - ruby
      - rails
    venue:
      city: Berlin
      country: DE
  "});

  assert_eq!(document.get_all("tags[]"), vec!["ruby", "rails"]);
  assert_eq!(document.get("venue.city"), Some("Berlin".to_string()));

  document.delete("tags[0]").unwrap();
  document.delete("venue.city").unwrap();

  assert_eq!(document.to_string(), "tags:\n  - rails\nvenue:\n  country: DE\n");
}
