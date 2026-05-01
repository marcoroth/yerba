use rowan::ast::AstNode;
use rowan::{TextRange, TextSize};
use yaml_parser::ast::{BlockMap, BlockMapEntry, BlockSeq, Root};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Document {
  root: SyntaxNode,
  path: Option<PathBuf>,
}

use crate::QuoteStyle;

#[derive(Debug)]
pub enum YerbaError {
  ParseError(String),
  IoError(std::io::Error),
  PathNotFound(String),
  NotASequence(String),
  IndexOutOfBounds(usize, usize),
  UnknownKeys(Vec<String>),
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

impl Document {
  pub fn parse(source: &str) -> Result<Self, YerbaError> {
    let tree =
      yaml_parser::parse(source).map_err(|error| YerbaError::ParseError(format!("{}", error)))?;

    Ok(Document {
      root: tree,
      path: None,
    })
  }

  pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, YerbaError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)?;
    let mut document = Self::parse(&source)?;
    document.path = Some(path.to_path_buf());
    Ok(document)
  }

  pub fn get(&self, dot_path: &str) -> Option<String> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys).ok()?;

    extract_scalar_text(&current_node)
  }

  pub fn set(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let scalar_token = find_scalar_token(&current_node)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let new_text = format_scalar_value(value, scalar_token.kind());

    self.replace_token(&scalar_token, &new_text)
  }

  pub fn append(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let last_entry = sequence
      .entries()
      .last()
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let indent = preceding_whitespace_indent(last_entry.syntax());
    let new_entry = format!("\n{}- {}", indent, value);

    self.insert_after_node(last_entry.syntax(), &new_entry)
  }

  pub fn rename(&mut self, dot_path: &str, new_key: &str) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let parent_node = self.navigate_to_path(&keys[..keys.len() - 1])?;
    let last_key = keys.last().unwrap();

    let map = parent_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entry = find_entry_by_key(&map, last_key)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let key_node = entry
      .key()
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let key_token = find_scalar_token(key_node.syntax())
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let new_text = format_scalar_value(new_key, key_token.kind());

    self.replace_token(&key_token, &new_text)
  }

  pub fn delete(&mut self, dot_path: &str) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let parent_node = self.navigate_to_path(&keys[..keys.len() - 1])?;
    let last_key = keys.last().unwrap();

    let map = parent_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entry = find_entry_by_key(&map, last_key)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    self.remove_node(entry.syntax())
  }

  pub fn remove(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let target_entry = sequence
      .entries()
      .find(|entry| {
        entry
          .flow()
          .and_then(|flow| extract_scalar_text(flow.syntax()))
          .map(|text| text == value)
          .unwrap_or(false)
      })
      .ok_or_else(|| YerbaError::PathNotFound(format!("{} item '{}'", dot_path, value)))?;

    self.remove_node(target_entry.syntax())
  }

  pub fn move_item(&mut self, dot_path: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();
    let length = entries.len();

    if from >= length {
      return Err(YerbaError::IndexOutOfBounds(from, length));
    }

    if to >= length {
      return Err(YerbaError::IndexOutOfBounds(to, length));
    }

    let entry_texts: Vec<String> = entries
      .iter()
      .map(|entry| entry.syntax().text().to_string())
      .collect();

    let mut reordered = entry_texts.clone();
    let item = reordered.remove(from);
    reordered.insert(to, item);

    let indent = entries
      .get(1)
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let sequence_text: String = reordered
      .iter()
      .enumerate()
      .map(|(index, text)| {
        if index == 0 {
          text.clone()
        } else {
          format!("\n{}{}", indent, text)
        }
      })
      .collect();

    let sequence_range = sequence.syntax().text_range();

    self.apply_edit(sequence_range, &sequence_text)
  }

  pub fn move_key(&mut self, dot_path: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entries: Vec<_> = map.entries().collect();
    let length = entries.len();

    if from >= length {
      return Err(YerbaError::IndexOutOfBounds(from, length));
    }

    if to >= length {
      return Err(YerbaError::IndexOutOfBounds(to, length));
    }

    let entry_texts: Vec<String> = entries
      .iter()
      .map(|entry| entry.syntax().text().to_string())
      .collect();

    let mut reordered = entry_texts.clone();
    let item = reordered.remove(from);
    reordered.insert(to, item);

    let indent = entries
      .get(1)
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let map_text: String = reordered
      .iter()
      .enumerate()
      .map(|(index, text)| {
        if index == 0 {
          text.clone()
        } else {
          format!("\n{}{}", indent, text)
        }
      })
      .collect();

    let map_range = map.syntax().text_range();

    self.apply_edit(map_range, &map_text)
  }

  pub fn resolve_key_index(&self, dot_path: &str, reference: &str) -> Result<usize, YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    if let Ok(index) = reference.parse::<usize>() {
      let length = map.entries().count();

      if index >= length {
        return Err(YerbaError::IndexOutOfBounds(index, length));
      }

      return Ok(index);
    }

    map
      .entries()
      .enumerate()
      .find(|(_index, entry)| {
        entry
          .key()
          .and_then(|key_node| extract_scalar_text(key_node.syntax()))
          .map(|key_text| key_text == reference)
          .unwrap_or(false)
      })
      .map(|(index, _entry)| index)
      .ok_or_else(|| YerbaError::PathNotFound(format!("{} key '{}'", dot_path, reference)))
  }

  pub fn resolve_sequence_index(
    &self,
    dot_path: &str,
    reference: &str,
  ) -> Result<usize, YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    if let Ok(index) = reference.parse::<usize>() {
      let length = sequence.entries().count();

      if index >= length {
        return Err(YerbaError::IndexOutOfBounds(index, length));
      }

      return Ok(index);
    }

    sequence
      .entries()
      .enumerate()
      .find(|(_index, entry)| {
        entry
          .flow()
          .and_then(|flow| extract_scalar_text(flow.syntax()))
          .map(|text| text == reference)
          .unwrap_or(false)
      })
      .map(|(index, _entry)| index)
      .ok_or_else(|| YerbaError::PathNotFound(format!("{} item '{}'", dot_path, reference)))
  }

  pub fn validate_sort_keys(&self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let unknown_keys: Vec<String> = map
      .entries()
      .filter_map(|entry| {
        entry
          .key()
          .and_then(|key_node| extract_scalar_text(key_node.syntax()))
      })
      .filter(|key_name| !key_order.contains(&key_name.as_str()))
      .collect();

    if unknown_keys.is_empty() {
      Ok(())
    } else {
      Err(YerbaError::UnknownKeys(unknown_keys))
    }
  }

  pub fn sort_keys(&mut self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entries: Vec<_> = map.entries().collect();

    if entries.len() <= 1 {
      return Ok(());
    }

    let entry_data: Vec<(String, String)> = entries
      .iter()
      .map(|entry| {
        let key_name = entry
          .key()
          .and_then(|key_node| extract_scalar_text(key_node.syntax()))
          .unwrap_or_default();
        let text = entry.syntax().text().to_string();
        (key_name, text)
      })
      .collect();

    let mut sorted = entry_data.clone();

    sorted.sort_by(|(key_a, _), (key_b, _)| {
      let position_a = key_order.iter().position(|&key| key == key_a);
      let position_b = key_order.iter().position(|&key| key == key_b);

      match (position_a, position_b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => {
          let original_a = entry_data.iter().position(|(key, _)| key == key_a).unwrap();
          let original_b = entry_data.iter().position(|(key, _)| key == key_b).unwrap();
          original_a.cmp(&original_b)
        }
      }
    });

    if sorted.iter().map(|(key, _)| key).collect::<Vec<_>>()
      == entry_data.iter().map(|(key, _)| key).collect::<Vec<_>>()
    {
      return Ok(());
    }

    let indent = entries
      .get(1)
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let map_text: String = sorted
      .iter()
      .enumerate()
      .map(|(index, (_key, text))| {
        if index == 0 {
          text.clone()
        } else {
          format!("\n{}{}", indent, text)
        }
      })
      .collect();

    let map_range = map.syntax().text_range();

    self.apply_edit(map_range, &map_text)
  }

  pub fn enforce_quotes(&mut self, style: &QuoteStyle) -> Result<(), YerbaError> {
    self.enforce_quotes_at(style, None)
  }

  pub fn enforce_quotes_at(
    &mut self,
    style: &QuoteStyle,
    dot_path: Option<&str>,
  ) -> Result<(), YerbaError> {
    let source = self.root.text().to_string();

    let scope_node = match dot_path {
      Some(path) if !path.is_empty() => {
        let keys: Vec<&str> = path.split('.').collect();
        self.navigate_to_path(&keys)?
      }
      _ => self.root.clone(),
    };

    let scope_range = scope_node.text_range();

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for element in self.root.descendants_with_tokens() {
      if let Some(token) = element.into_token() {
        if !scope_range.contains_range(token.text_range()) {
          continue;
        }

        if is_map_key(&token) {
          continue;
        }

        let current_kind = token.kind();

        if !matches!(
          current_kind,
          SyntaxKind::PLAIN_SCALAR
            | SyntaxKind::DOUBLE_QUOTED_SCALAR
            | SyntaxKind::SINGLE_QUOTED_SCALAR
        ) {
          continue;
        }

        let target_kind = style.to_syntax_kind();

        if current_kind == target_kind {
          continue;
        }

        let raw_value = match current_kind {
          SyntaxKind::DOUBLE_QUOTED_SCALAR => {
            let text = token.text();
            text[1..text.len() - 1].to_string()
          }
          SyntaxKind::SINGLE_QUOTED_SCALAR => {
            let text = token.text();
            text[1..text.len() - 1].to_string()
          }
          SyntaxKind::PLAIN_SCALAR => token.text().to_string(),
          _ => continue,
        };

        let new_text = match style {
          QuoteStyle::DoubleQuoted => format!("\"{}\"", raw_value),
          QuoteStyle::SingleQuoted => format!("'{}'", raw_value),
          QuoteStyle::Plain => raw_value,
          _ => continue,
        };

        if new_text != token.text() {
          edits.push((token.text_range(), new_text));
        }
      }
    }

    if edits.is_empty() {
      return Ok(());
    }

    edits.reverse();

    let mut new_source = source;

    for (range, replacement) in edits {
      let start: usize = range.start().into();
      let end: usize = range.end().into();

      new_source.replace_range(start..end, &replacement);
    }

    let path = self.path.take();
    *self = Self::parse(&new_source)?;
    self.path = path;

    Ok(())
  }

  pub fn save(&self) -> Result<(), YerbaError> {
    let path = self.path.as_ref().ok_or_else(|| {
      YerbaError::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "no file path associated with this document",
      ))
    })?;

    fs::write(path, self.to_string())?;

    Ok(())
  }

  pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), YerbaError> {
    fs::write(path, self.to_string())?;

    Ok(())
  }

  fn navigate_to_path(&self, keys: &[&str]) -> Result<SyntaxNode, YerbaError> {
    let keys: Vec<&&str> = keys.iter().filter(|key| !key.is_empty()).collect();
    let path_string = keys.iter().map(|key| **key).collect::<Vec<_>>().join(".");

    let root =
      Root::cast(self.root.clone()).ok_or_else(|| YerbaError::PathNotFound(path_string.clone()))?;

    let document = root
      .documents()
      .next()
      .ok_or_else(|| YerbaError::PathNotFound(path_string.clone()))?;

    let mut current_node = document.syntax().clone();

    for key in &keys {
      let map = current_node
        .descendants()
        .find_map(BlockMap::cast)
        .ok_or_else(|| YerbaError::PathNotFound(path_string.clone()))?;

      let entry = find_entry_by_key(&map, key)
        .ok_or_else(|| YerbaError::PathNotFound(path_string.clone()))?;

      let map_value = entry
        .value()
        .ok_or_else(|| YerbaError::PathNotFound(path_string.clone()))?;

      current_node = map_value.syntax().clone();
    }

    Ok(current_node)
  }

  fn replace_token(&mut self, token: &SyntaxToken, new_text: &str) -> Result<(), YerbaError> {
    let range = token.text_range();

    self.apply_edit(range, new_text)
  }

  fn insert_after_node(&mut self, node: &SyntaxNode, text: &str) -> Result<(), YerbaError> {
    let position = node.text_range().end();
    let range = TextRange::new(position, position);

    self.apply_edit(range, text)
  }

  fn remove_node(&mut self, node: &SyntaxNode) -> Result<(), YerbaError> {
    let range = removal_range(node);

    self.apply_edit(range, "")
  }

  fn apply_edit(&mut self, range: TextRange, replacement: &str) -> Result<(), YerbaError> {
    let source = self.root.text().to_string();
    let start: usize = range.start().into();
    let end: usize = range.end().into();

    let mut new_source = source;
    new_source.replace_range(start..end, replacement);

    let path = self.path.take();
    *self = Self::parse(&new_source)?;
    self.path = path;

    Ok(())
  }
}

impl std::fmt::Display for Document {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.root.text())
  }
}

fn is_map_key(token: &SyntaxToken) -> bool {
  token
    .parent_ancestors()
    .any(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_KEY)
}

fn find_entry_by_key(map: &BlockMap, key: &str) -> Option<BlockMapEntry> {
  map.entries().find(|entry| {
    entry
      .key()
      .and_then(|key_node| extract_scalar_text(key_node.syntax()))
      .map(|key_text| key_text == key)
      .unwrap_or(false)
  })
}

fn find_scalar_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .descendants_with_tokens()
    .filter_map(|element| element.into_token())
    .find(|token| {
      matches!(
        token.kind(),
        SyntaxKind::PLAIN_SCALAR
          | SyntaxKind::DOUBLE_QUOTED_SCALAR
          | SyntaxKind::SINGLE_QUOTED_SCALAR
      )
    })
}

fn format_scalar_value(value: &str, kind: SyntaxKind) -> String {
  match kind {
    SyntaxKind::DOUBLE_QUOTED_SCALAR => format!("\"{}\"", value),
    SyntaxKind::SINGLE_QUOTED_SCALAR => format!("'{}'", value),
    _ => value.to_string(),
  }
}

fn preceding_whitespace_indent(node: &SyntaxNode) -> String {
  preceding_whitespace_token(node)
    .map(|token| {
      let text = token.text();

      text
        .rfind('\n')
        .map(|newline| text[newline + 1..].to_string())
        .unwrap_or_default()
    })
    .unwrap_or_default()
}

fn preceding_whitespace_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .prev_sibling_or_token()
    .and_then(|sibling| sibling.into_token())
    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
}

fn following_whitespace_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .next_sibling_or_token()
    .and_then(|sibling| sibling.into_token())
    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
}

fn removal_range(node: &SyntaxNode) -> TextRange {
  let node_range = node.text_range();

  if let Some(whitespace_token) = preceding_whitespace_token(node) {
    let whitespace_text = whitespace_token.text();
    let whitespace_start = whitespace_token.text_range().start();

    let remove_from = whitespace_text
      .rfind('\n')
      .map(|offset| whitespace_start + TextSize::from(offset as u32))
      .unwrap_or(whitespace_start);

    return TextRange::new(remove_from, node_range.end());
  }

  if let Some(whitespace_token) = following_whitespace_token(node) {
    return TextRange::new(node_range.start(), whitespace_token.text_range().end());
  }

  node_range
}

fn extract_scalar_text(node: &SyntaxNode) -> Option<String> {
  let token = find_scalar_token(node)?;

  match token.kind() {
    SyntaxKind::PLAIN_SCALAR => Some(token.text().to_string()),

    SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let text = token.text();

      Some(text[1..text.len() - 1].to_string())
    }

    _ => None,
  }
}
