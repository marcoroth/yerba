mod support;
use support::parse;
use yerba::find_control_characters;

#[test]
fn test_find_control_characters_reports_line_and_column() {
  let source = "---\n- title: \"abc\u{8}def\"\n  id: x\n";

  let found = find_control_characters(source);

  assert_eq!(found.len(), 1);
  assert_eq!(found[0].character, '\u{8}');
  assert_eq!(found[0].line, 2);
  assert_eq!(found[0].column, 14);
}

#[test]
fn test_find_control_characters_reports_every_occurrence() {
  let source = "a: x\u{8}y\nb: ok\nc: p\u{1b}q\u{7f}r\n";

  let found = find_control_characters(source);

  let lines: Vec<usize> = found.iter().map(|control| control.line).collect();

  assert_eq!(lines, vec![1, 3, 3]);
}

#[test]
fn test_find_control_characters_ignores_escaped_forms() {
  let source = "a: \"abc\\x08def\"\nb: \"tab\\there\"\n";

  assert!(find_control_characters(source).is_empty());
}

#[test]
fn test_find_control_characters_accepts_a_clean_document() {
  let source = "---\n- title: Hello\n  tags:\n    - ruby\n";

  assert!(find_control_characters(source).is_empty());
}

#[test]
fn test_a_written_value_never_leaves_a_literal_control_character_behind() {
  let mut document = parse("title: Hello\n");

  document.set("title", "abc\u{8}def").unwrap();

  assert!(find_control_characters(&document.to_string()).is_empty());
}

#[cfg(feature = "cli")]
mod check {
  use indoc::indoc;
  use std::fs;
  use tempfile::TempDir;

  fn yerbafile_in(dir: &TempDir) -> yerba::Yerbafile {
    fs::write(
      dir.path().join("Yerbafile"),
      indoc! {r#"
        rules:
          - files: "*.yml"
            pipeline:
              - collection_style:
                  style: block
      "#},
    )
    .unwrap();

    yerba::Yerbafile::load(dir.path().join("Yerbafile")).unwrap()
  }

  #[test]
  fn test_apply_rejects_a_file_with_control_characters() {
    let dir = TempDir::new().unwrap();
    let yerbafile = yerbafile_in(&dir);

    let file = dir.path().join("talks.yml");
    fs::write(&file, "- title: \"abc\u{8}def\"\n  id: x\n").unwrap();

    let results = yerbafile.apply_file(file.to_str().unwrap(), false);
    let error = results[0].error.as_ref().expect("expected a control character error").to_string();

    assert!(error.contains("control character"), "{}", error);
    assert!(error.contains("U+0008 at line 1, column 14"), "{}", error);
  }

  #[test]
  fn test_apply_accepts_a_file_whose_control_characters_are_escaped() {
    let dir = TempDir::new().unwrap();
    let yerbafile = yerbafile_in(&dir);

    let file = dir.path().join("talks.yml");
    fs::write(&file, "- title: \"abc\\x08def\"\n  id: x\n").unwrap();

    let results = yerbafile.apply_file(file.to_str().unwrap(), false);

    assert!(results[0].error.is_none());
  }
}
