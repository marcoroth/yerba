use crate::syntax::is_control_character;

#[derive(Debug, Clone)]
pub struct ControlCharacter {
  pub character: char,
  pub line: usize,
  pub column: usize,
  pub character_column: usize,
}

impl std::fmt::Display for ControlCharacter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "U+{:04X} at line {}, column {}", self.character as u32, self.line, self.character_column + 1)
  }
}

pub fn find_control_characters(source: &str) -> Vec<ControlCharacter> {
  source
    .lines()
    .enumerate()
    .flat_map(|(index, line)| {
      line.char_indices().enumerate().filter_map(move |(character_column, (column, character))| {
        is_control_character(character).then_some(ControlCharacter {
          character,
          line: index + 1,
          column,
          character_column,
        })
      })
    })
    .collect()
}
