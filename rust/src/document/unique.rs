use rowan::ast::AstNode;

use crate::document::{extract_scalar_text, navigate_from_node, Document};
use crate::error::YerbaError;
use crate::syntax::{find_block_sequence, line_at};

#[derive(Debug, Clone)]
pub struct DuplicateInfo {
  pub value: String,
  pub line: usize,
}

impl Document {
  pub fn unique(&mut self, dot_path: &str, by: &str, remove: bool) -> Result<Vec<DuplicateInfo>, YerbaError> {
    self.unique_with_options(dot_path, by, remove, false)
  }

  pub fn unique_with_options(&mut self, dot_path: &str, by: &str, remove: bool, allow_blank_duplicates: bool) -> Result<Vec<DuplicateInfo>, YerbaError> {
    let current_node = self.navigate(dot_path)?;
    let source = self.source_text();

    let sequence = match find_block_sequence(&current_node) {
      Some(sequence) => sequence,
      None => return Ok(Vec::new()),
    };

    let entries: Vec<_> = sequence.entries().collect();

    if entries.len() <= 1 {
      return Ok(Vec::new());
    }

    let by_is_scalar = by == ".";

    let labels: Vec<(Option<String>, usize)> = entries
      .iter()
      .map(|entry| {
        let offset: usize = entry.syntax().text_range().start().into();
        let line = line_at(&source, offset);

        let value = if by_is_scalar {
          Some(entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())).unwrap_or_default())
        } else {
          let field = by.strip_prefix('.').unwrap_or(by);
          let nodes = navigate_from_node(entry.syntax(), field);

          nodes.first().and_then(extract_scalar_text)
        };

        (value, line)
      })
      .collect();

    let mut seen = std::collections::HashSet::new();
    let mut duplicate_indices: Vec<usize> = Vec::new();
    let mut duplicates: Vec<DuplicateInfo> = Vec::new();

    for (index, (label, line)) in labels.iter().enumerate() {
      let label = match label {
        None => continue,
        Some(value) => value,
      };

      if allow_blank_duplicates && label.is_empty() {
        continue;
      }

      if !seen.insert(label.clone()) {
        duplicate_indices.push(index);
        duplicates.push(DuplicateInfo {
          value: label.clone(),
          line: *line,
        });
      }
    }

    if remove && !duplicate_indices.is_empty() {
      for &index in duplicate_indices.iter().rev() {
        self.remove_at(dot_path, index)?;
      }
    }

    Ok(duplicates)
  }
}
