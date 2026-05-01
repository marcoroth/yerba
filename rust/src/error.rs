#[derive(Debug)]
pub enum YerbaError {
  ParseError(String),
  IoError(std::io::Error),
  PathNotFound(String),
  NotASequence(String),
  IndexOutOfBounds(usize, usize),
  UnknownKeys(Vec<String>),
  ReferenceNotFound(String),
}

impl std::fmt::Display for YerbaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      YerbaError::ParseError(msg) => write!(f, "parse error: {}", msg),
      YerbaError::IoError(err) => write!(f, "io error: {}", err),
      YerbaError::PathNotFound(path) => write!(f, "path not found: {}", path),
      YerbaError::NotASequence(path) => write!(f, "not a sequence: {}", path),

      YerbaError::IndexOutOfBounds(index, length) => {
        write!(f, "index {} out of bounds (length {})", index, length)
      }

      YerbaError::ReferenceNotFound(reference) => {
        write!(f, "template reference not found: ${{{}}}", reference)
      }

      YerbaError::UnknownKeys(keys) => {
        let suggestion = keys
          .iter()
          .map(|key| format!("\"{}\"", key))
          .collect::<Vec<_>>()
          .join(", ");

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
