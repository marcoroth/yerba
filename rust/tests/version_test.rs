use yerba::version;

#[test]
fn test_version() {
  assert_eq!(version(), "0.4.0");
}
