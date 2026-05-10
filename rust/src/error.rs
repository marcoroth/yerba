#[derive(Debug)]
pub enum YerbaError {
  ParseError(String),
  IoError(std::io::Error),
  SelectorNotFound(String),
  AmbiguousSelector(String, usize),
  NotASequence(String),
  IndexOutOfBounds(usize, usize),
  UnknownKeys(Vec<String>),
  DuplicateValues(Vec<crate::DuplicateInfo>),
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

      YerbaError::DuplicateValues(duplicates) => {
        let noun = if duplicates.len() == 1 { "duplicate" } else { "duplicates" };
        let details: Vec<String> = duplicates
          .iter()
          .map(|duplicate| format!("\"{}\" (line {})", duplicate.value, duplicate.line))
          .collect();

        write!(f, "found {} {}: {}", duplicates.len(), noun, details.join(", "))
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

pub struct GitHubAnnotation {
  pub level: &'static str,
  pub file: String,
  pub line: Option<usize>,
  pub message: String,
}

impl std::fmt::Display for GitHubAnnotation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.line {
      Some(line) => write!(f, "::{}file={},line={}::{}", self.level, self.file, line, self.message),
      None => write!(f, "::{}file={}::{}", self.level, self.file, self.message),
    }
  }
}

pub trait GitHubAnnotations {
  fn github_annotations(&self, file: &str) -> Vec<GitHubAnnotation>;
}

impl GitHubAnnotations for YerbaError {
  fn github_annotations(&self, file: &str) -> Vec<GitHubAnnotation> {
    match self {
      YerbaError::DuplicateValues(duplicates) => duplicates
        .iter()
        .map(|duplicate| GitHubAnnotation {
          level: "error ",
          file: file.to_string(),
          line: Some(duplicate.line),
          message: format!("duplicate: \"{}\"", duplicate.value),
        })
        .collect(),

      YerbaError::DuplicateKey { key, duplicate_line, .. } => vec![GitHubAnnotation {
        level: "error ",
        file: file.to_string(),
        line: Some(*duplicate_line),
        message: format!("duplicate key: \"{}\"", key),
      }],

      _ => vec![GitHubAnnotation {
        level: "error ",
        file: file.to_string(),
        line: None,
        message: self.to_string(),
      }],
    }
  }
}
