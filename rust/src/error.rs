#[derive(Debug)]
pub enum YerbaError {
  ParseError(String),
  IoError(std::io::Error),
  SelectorNotFound(String),
  AmbiguousSelector(String, usize),
  NotASequence(String),
  IndexOutOfBounds(usize, usize),
  UnknownKeys(Vec<String>),
  DuplicateKey {
    key: String,
    first_line: usize,
    duplicate_line: usize,
    line_content: String,
  },
}

impl std::fmt::Display for YerbaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      YerbaError::ParseError(msg) => write!(f, "parse error: {}", msg),
      YerbaError::IoError(err) => write!(f, "io error: {}", err),
      YerbaError::SelectorNotFound(selector) => write!(f, "selector not found: {}", selector),
      YerbaError::NotASequence(path) => write!(f, "not a sequence: {}", path),

      YerbaError::AmbiguousSelector(selector, count) => {
        write!(
          f,
          "selector \"{}\" matched {} nodes (expected 1). Use --all to update all matches",
          selector, count
        )
      }

      YerbaError::DuplicateKey {
        key,
        first_line,
        duplicate_line,
        line_content,
      } => {
        write!(
          f,
          "duplicate key \"{}\" on line {} (first defined on line {})\n\n    {} | {}",
          key,
          duplicate_line,
          first_line,
          duplicate_line,
          line_content.trim()
        )
      }

      YerbaError::IndexOutOfBounds(index, length) => {
        write!(f, "index {} out of bounds (length {})", index, length)
      }

      YerbaError::UnknownKeys(keys) => {
        let suggestion = keys.iter().map(|key| format!("\"{}\"", key)).collect::<Vec<_>>().join(", ");

        write!(
          f,
          "found keys not listed in sort order: {}\n\n  Add them to your sort order or Yerbafile:\n    {}\n",
          keys.join(", "),
          suggestion
        )
      }
    }
  }
}

impl From<std::io::Error> for YerbaError {
  fn from(err: std::io::Error) -> Self {
    YerbaError::IoError(err)
  }
}
