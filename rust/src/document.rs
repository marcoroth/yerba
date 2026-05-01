use std::fs;
use std::path::{Path, PathBuf};

use rowan::ast::AstNode;
use rowan::TextRange;

use yaml_parser::ast::{BlockMap, BlockSeq, Root};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

use crate::error::YerbaError;
use crate::QuoteStyle;

use crate::syntax::{
  extract_scalar_text, find_entry_by_key, find_scalar_token, format_scalar_value, is_map_key,
  is_yaml_non_string, preceding_whitespace_indent, preceding_whitespace_token, removal_range,
  unescape_double_quoted, unescape_single_quoted,
};

#[derive(Debug)]
pub struct FindResult {
  pub text: String,
  pub line: usize,
  pub end_line: usize,
}

#[derive(Debug)]
pub enum InsertPosition {
  At(usize),
  Last,
  Before(String),
  After(String),
  FromSortOrder(Vec<String>),
}

#[derive(Debug)]
pub struct Document {
  root: SyntaxNode,
  path: Option<PathBuf>,
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
    if dot_path.contains('[') {
      return self.get_all(dot_path).into_iter().next();
    }

    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys).ok()?;

    extract_scalar_text(&current_node)
  }

  pub fn get_all(&self, dot_path: &str) -> Vec<String> {
    self
      .navigate_to_many(dot_path)
      .iter()
      .filter_map(extract_scalar_text)
      .collect()
  }

  pub fn find_items(&self, dot_path: &str, condition: &str) -> Vec<FindResult> {
    let source = self.root.text().to_string();
    let nodes = self.navigate_to_many(dot_path);

    nodes
      .iter()
      .filter(|node| self.evaluate_condition_on_node(node, condition))
      .map(|node| {
        let start_offset: usize = node.text_range().start().into();
        let end_offset: usize = node.text_range().end().into();

        FindResult {
          text: node.text().to_string(),
          line: byte_offset_to_line(&source, start_offset),
          end_line: byte_offset_to_line(&source, end_offset),
        }
      })
      .collect()
  }

  pub fn find_all(&self, dot_path: &str) -> Vec<FindResult> {
    let source = self.root.text().to_string();

    self
      .navigate_to_many(dot_path)
      .iter()
      .map(|node| {
        let start_offset: usize = node.text_range().start().into();
        let end_offset: usize = node.text_range().end().into();

        FindResult {
          text: node.text().to_string(),
          line: byte_offset_to_line(&source, start_offset),
          end_line: byte_offset_to_line(&source, end_offset),
        }
      })
      .collect()
  }

  fn evaluate_condition_on_node(&self, node: &SyntaxNode, condition: &str) -> bool {
    let condition = condition.trim();

    let (left, operator, right) = match parse_condition(condition) {
      Some(parts) => parts,
      None => return false,
    };

    let left_path = left.strip_prefix('.').unwrap_or(&left);
    let target_nodes = navigate_from_node(node, left_path);

    let values: Vec<String> = target_nodes
      .iter()
      .filter_map(extract_scalar_text)
      .collect();

    match operator {
      "==" => values.iter().any(|value| value == &right),
      "!=" => values.iter().all(|value| value != &right),
      "contains" => {
        if values
          .iter()
          .any(|value| value == &right || value.contains(&right))
        {
          return true;
        }

        for node in &target_nodes {
          if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
            for entry in sequence.entries() {
              if let Some(text) = entry
                .flow()
                .and_then(|flow| extract_scalar_text(flow.syntax()))
              {
                if text == right {
                  return true;
                }
              }
            }
          }
        }

        false
      }
      "not_contains" => {
        for node in &target_nodes {
          if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
            for entry in sequence.entries() {
              if let Some(text) = entry
                .flow()
                .and_then(|flow| extract_scalar_text(flow.syntax()))
              {
                if text == right {
                  return false;
                }
              }
            }
          }
        }

        !values
          .iter()
          .any(|value| value == &right || value.contains(&right))
      }
      _ => false,
    }
  }

  pub fn exists(&self, dot_path: &str) -> bool {
    if dot_path.contains('[') {
      return !self.navigate_to_many(dot_path).is_empty();
    }

    self.get(dot_path).is_some()
  }

  pub fn evaluate_condition(&self, parent_path: &str, condition: &str) -> bool {
    let condition = condition.trim();

    let (left, operator, right) = match parse_condition(condition) {
      Some(parts) => parts,
      None => return false,
    };

    let full_path = if let Some(relative_key) = left.strip_prefix('.') {
      if parent_path.is_empty() {
        relative_key.to_string()
      } else {
        format!("{}.{}", parent_path, relative_key)
      }
    } else {
      left.to_string()
    };

    let has_brackets = full_path.contains('[');

    match operator {
      "==" => {
        if has_brackets {
          self.get_all(&full_path).iter().any(|value| value == &right)
        } else {
          self.get(&full_path).unwrap_or_default() == right
        }
      }
      "!=" => {
        if has_brackets {
          self.get_all(&full_path).iter().all(|value| value != &right)
        } else {
          self.get(&full_path).unwrap_or_default() != right
        }
      }
      "contains" => {
        if has_brackets {
          self
            .get_all(&full_path)
            .iter()
            .any(|value| value == &right || value.contains(&right))
        } else {
          let items = self.get_sequence_values(&full_path);

          if !items.is_empty() {
            items.iter().any(|item| item == &right)
          } else {
            self
              .get(&full_path)
              .map(|value| value.contains(&right))
              .unwrap_or(false)
          }
        }
      }
      "not_contains" => {
        if has_brackets {
          self
            .get_all(&full_path)
            .iter()
            .all(|value| value != &right && !value.contains(&right))
        } else {
          let items = self.get_sequence_values(&full_path);

          if !items.is_empty() {
            !items.iter().any(|item| item == &right)
          } else {
            self
              .get(&full_path)
              .map(|value| !value.contains(&right))
              .unwrap_or(true)
          }
        }
      }
      _ => false,
    }
  }

  pub fn get_sequence_values(&self, dot_path: &str) -> Vec<String> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = match self.navigate_to_path(&keys) {
      Ok(node) => node,
      Err(_) => return Vec::new(),
    };

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Vec::new(),
    };

    sequence
      .entries()
      .filter_map(|entry| {
        entry
          .flow()
          .and_then(|flow| extract_scalar_text(flow.syntax()))
      })
      .collect()
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
    self.insert_into(dot_path, value, InsertPosition::Last)
  }

  pub fn insert_into(
    &mut self,
    dot_path: &str,
    value: &str,
    position: InsertPosition,
  ) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();

    if let Ok(current_node) = self.navigate_to_path(&keys) {
      if current_node
        .descendants()
        .find_map(BlockSeq::cast)
        .is_some()
      {
        return self.insert_sequence_item(dot_path, value, position);
      }
    }

    let (parent_path, key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));

    self.insert_map_key(parent_path, key, value, position)
  }

  fn insert_sequence_item(
    &mut self,
    dot_path: &str,
    value: &str,
    position: InsertPosition,
  ) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    if entries.is_empty() {
      return Err(YerbaError::PathNotFound(dot_path.to_string()));
    }

    let indent = entries
      .get(1)
      .or(entries.first())
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let new_item = format!("- {}", value);

    match position {
      InsertPosition::Last => {
        let last_entry = entries.last().unwrap();
        let new_text = format!("\n{}{}", indent, new_item);

        self.insert_after_node(last_entry.syntax(), &new_text)
      }

      InsertPosition::At(index) => {
        if index >= entries.len() {
          let last_entry = entries.last().unwrap();
          let new_text = format!("\n{}{}", indent, new_item);

          self.insert_after_node(last_entry.syntax(), &new_text)
        } else {
          let target_entry = &entries[index];
          let target_range = target_entry.syntax().text_range();
          let replacement = format!("{}\n{}", new_item, indent);
          let insert_range = TextRange::new(target_range.start(), target_range.start());

          self.apply_edit(insert_range, &replacement)
        }
      }

      InsertPosition::Before(target_value) => {
        let target_entry = entries
          .iter()
          .find(|entry| {
            entry
              .flow()
              .and_then(|flow| extract_scalar_text(flow.syntax()))
              .map(|text| text == target_value)
              .unwrap_or(false)
          })
          .ok_or_else(|| {
            YerbaError::PathNotFound(format!("{} item '{}'", dot_path, target_value))
          })?;

        let target_range = target_entry.syntax().text_range();
        let replacement = format!("{}\n{}", new_item, indent);
        let insert_range = TextRange::new(target_range.start(), target_range.start());

        self.apply_edit(insert_range, &replacement)
      }

      InsertPosition::After(target_value) => {
        let target_entry = entries
          .iter()
          .find(|entry| {
            entry
              .flow()
              .and_then(|flow| extract_scalar_text(flow.syntax()))
              .map(|text| text == target_value)
              .unwrap_or(false)
          })
          .ok_or_else(|| {
            YerbaError::PathNotFound(format!("{} item '{}'", dot_path, target_value))
          })?;

        let new_text = format!("\n{}{}", indent, new_item);

        self.insert_after_node(target_entry.syntax(), &new_text)
      }

      InsertPosition::FromSortOrder(_) => {
        let last_entry = entries.last().unwrap();
        let new_text = format!("\n{}{}", indent, new_item);

        self.insert_after_node(last_entry.syntax(), &new_text)
      }
    }
  }

  fn insert_map_key(
    &mut self,
    dot_path: &str,
    key: &str,
    value: &str,
    position: InsertPosition,
  ) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entries: Vec<_> = map.entries().collect();

    if entries.is_empty() {
      let indent = preceding_whitespace_indent(map.syntax());
      let new_entry = format!("\n{}{}: {}", indent, key, value);

      return self.insert_after_node(map.syntax(), &new_entry);
    }

    if find_entry_by_key(&map, key).is_some() {
      return Err(YerbaError::ParseError(format!(
        "key '{}' already exists at '{}'",
        key, dot_path
      )));
    }

    let indent = entries
      .get(1)
      .or(entries.first())
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let new_entry_text = format!("{}: {}", key, value);

    match position {
      InsertPosition::Last => {
        let last_entry = entries.last().unwrap();
        let new_text = format!("\n{}{}", indent, new_entry_text);

        self.insert_after_node(last_entry.syntax(), &new_text)
      }

      InsertPosition::At(index) => {
        if index >= entries.len() {
          let last_entry = entries.last().unwrap();
          let new_text = format!("\n{}{}", indent, new_entry_text);

          self.insert_after_node(last_entry.syntax(), &new_text)
        } else {
          let target_entry = &entries[index];
          let target_range = target_entry.syntax().text_range();
          let replacement = format!("{}\n{}", new_entry_text, indent);
          let insert_range = TextRange::new(target_range.start(), target_range.start());

          self.apply_edit(insert_range, &replacement)
        }
      }

      InsertPosition::Before(target_key) => {
        let target_entry = find_entry_by_key(&map, &target_key)
          .ok_or_else(|| YerbaError::PathNotFound(format!("{}.{}", dot_path, target_key)))?;

        let target_range = target_entry.syntax().text_range();
        let replacement = format!("{}\n{}", new_entry_text, indent);
        let insert_range = TextRange::new(target_range.start(), target_range.start());

        self.apply_edit(insert_range, &replacement)
      }

      InsertPosition::After(target_key) => {
        let target_entry = find_entry_by_key(&map, &target_key)
          .ok_or_else(|| YerbaError::PathNotFound(format!("{}.{}", dot_path, target_key)))?;

        let new_text = format!("\n{}{}", indent, new_entry_text);

        self.insert_after_node(target_entry.syntax(), &new_text)
      }

      InsertPosition::FromSortOrder(order) => {
        let new_key_position = order.iter().position(|ordered_key| ordered_key == key);

        let resolved = match new_key_position {
          Some(new_position) => {
            let mut insert_after: Option<String> = None;

            for ordered_key in order.iter().take(new_position).rev() {
              if find_entry_by_key(&map, ordered_key).is_some() {
                insert_after = Some(ordered_key.clone());
                break;
              }
            }

            match insert_after {
              Some(after_key) => InsertPosition::After(after_key),
              None => InsertPosition::At(0),
            }
          }

          None => InsertPosition::Last,
        };

        self.insert_map_key(dot_path, key, value, resolved)
      }
    }
  }

  pub fn rename(&mut self, source_path: &str, destination_path: &str) -> Result<(), YerbaError> {
    let source_parent = source_path
      .rsplit_once('.')
      .map(|(parent, _)| parent)
      .unwrap_or("");

    let destination_parent = destination_path
      .rsplit_once('.')
      .map(|(parent, _)| parent)
      .unwrap_or("");

    let destination_key = destination_path
      .rsplit_once('.')
      .map(|(_, key)| key)
      .unwrap_or(destination_path);

    if source_parent == destination_parent {
      let keys: Vec<&str> = source_path.split('.').collect();
      let parent_node = self.navigate_to_path(&keys[..keys.len() - 1])?;
      let source_key = keys.last().unwrap();

      let map = parent_node
        .descendants()
        .find_map(BlockMap::cast)
        .ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      let entry = find_entry_by_key(&map, source_key)
        .ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      let key_node = entry
        .key()
        .ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      let key_token = find_scalar_token(key_node.syntax())
        .ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      let new_text = format_scalar_value(destination_key, key_token.kind());

      self.replace_token(&key_token, &new_text)
    } else {
      let value = self
        .get(source_path)
        .ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      self.delete(source_path)?;
      self.insert_into(destination_path, &value, InsertPosition::Last)
    }
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

    self.reorder_entries(
      &entries,
      from,
      to,
      |entry| entry.syntax().text().to_string(),
      |entry| preceding_whitespace_indent(entry.syntax()),
      sequence.syntax().text_range(),
    )
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

    self.reorder_entries(
      &entries,
      from,
      to,
      |entry| entry.syntax().text().to_string(),
      |entry| preceding_whitespace_indent(entry.syntax()),
      map.syntax().text_range(),
    )
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
    if dot_path == "[]" || dot_path.ends_with(".[]") {
      let seq_path = if dot_path == "[]" {
        ""
      } else {
        &dot_path[..dot_path.len() - 3]
      };

      return self.validate_each_sort_keys(seq_path, key_order);
    }

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
    if dot_path == "[]" || dot_path.ends_with(".[]") {
      let seq_path = if dot_path == "[]" {
        ""
      } else {
        &dot_path[..dot_path.len() - 3]
      };

      return self.sort_each_keys(seq_path, key_order);
    }

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

    let map_text = rebuild_entries(sorted.iter().map(|(_key, text)| text.as_str()), &indent);
    let map_range = map.syntax().text_range();

    self.apply_edit(map_range, &map_text)
  }

  pub fn sort_each_keys(&mut self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for entry in sequence.entries() {
      let entry_node = entry.syntax();

      let map = match entry_node.descendants().find_map(BlockMap::cast) {
        Some(map) => map,
        None => continue,
      };

      let entries: Vec<_> = map.entries().collect();

      if entries.len() <= 1 {
        continue;
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
        continue;
      }

      let indent = entries
        .get(1)
        .map(|entry| preceding_whitespace_indent(entry.syntax()))
        .unwrap_or_default();

      let map_text = rebuild_entries(sorted.iter().map(|(_key, text)| text.as_str()), &indent);

      edits.push((map.syntax().text_range(), map_text));
    }

    if edits.is_empty() {
      return Ok(());
    }

    edits.reverse();

    let source = self.root.text().to_string();
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

  pub fn validate_each_sort_keys(
    &self,
    dot_path: &str,
    key_order: &[&str],
  ) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    let mut all_unknown: Vec<String> = Vec::new();

    for entry in sequence.entries() {
      if let Some(map) = entry.syntax().descendants().find_map(BlockMap::cast) {
        for map_entry in map.entries() {
          if let Some(key_name) = map_entry
            .key()
            .and_then(|key_node| extract_scalar_text(key_node.syntax()))
          {
            if !key_order.contains(&key_name.as_str()) && !all_unknown.contains(&key_name) {
              all_unknown.push(key_name);
            }
          }
        }
      }
    }

    if all_unknown.is_empty() {
      Ok(())
    } else {
      Err(YerbaError::UnknownKeys(all_unknown))
    }
  }

  pub fn enforce_blank_lines(
    &mut self,
    dot_path: &str,
    blank_lines: usize,
  ) -> Result<(), YerbaError> {
    let keys: Vec<&str> = dot_path.split('.').collect();
    let current_node = self.navigate_to_path(&keys)?;

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    let entries: Vec<_> = sequence.entries().collect();

    if entries.len() <= 1 {
      return Ok(());
    }

    let source = self.root.text().to_string();
    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for entry in entries.iter().skip(1) {
      if let Some(whitespace_token) = preceding_whitespace_token(entry.syntax()) {
        let whitespace_text = whitespace_token.text();

        let newline_count = whitespace_text
          .chars()
          .filter(|character| *character == '\n')
          .count();

        let indent = whitespace_text
          .rfind('\n')
          .map(|position| &whitespace_text[position + 1..])
          .unwrap_or("");

        let desired_newlines = blank_lines + 1;

        if newline_count != desired_newlines {
          let new_whitespace = format!("{}{}", "\n".repeat(desired_newlines), indent);

          edits.push((whitespace_token.text_range(), new_whitespace));
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

  pub fn enforce_key_style(
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

        if !is_map_key(&token) {
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
            unescape_double_quoted(&text[1..text.len() - 1])
          }

          SyntaxKind::SINGLE_QUOTED_SCALAR => {
            let text = token.text();
            unescape_single_quoted(&text[1..text.len() - 1])
          }

          SyntaxKind::PLAIN_SCALAR => token.text().to_string(),

          _ => continue,
        };

        let new_text = match style {
          QuoteStyle::Double => {
            let escaped = raw_value.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
          }

          QuoteStyle::Single => {
            let escaped = raw_value.replace('\'', "''");
            format!("'{}'", escaped)
          }

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
            unescape_double_quoted(&text[1..text.len() - 1])
          }

          SyntaxKind::SINGLE_QUOTED_SCALAR => {
            let text = token.text();
            unescape_single_quoted(&text[1..text.len() - 1])
          }

          SyntaxKind::PLAIN_SCALAR => token.text().to_string(),

          _ => continue,
        };

        if is_yaml_non_string(&raw_value) {
          continue;
        }

        let new_text = match style {
          QuoteStyle::Double => {
            if raw_value.contains('"') && current_kind == SyntaxKind::SINGLE_QUOTED_SCALAR {
              continue;
            }

            let escaped = raw_value.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
          }

          QuoteStyle::Single => {
            let escaped = raw_value.replace('\'', "''");
            format!("'{}'", escaped)
          }

          QuoteStyle::Plain => {
            if raw_value.contains('"')
              || raw_value.contains('\'')
              || raw_value.contains(':')
              || raw_value.contains('#')
            {
              continue;
            }

            raw_value
          }

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

  pub fn navigate_to_many(&self, dot_path: &str) -> Vec<SyntaxNode> {
    let segments = parse_path_segments(dot_path);

    let root = match Root::cast(self.root.clone()) {
      Some(root) => root,
      None => return Vec::new(),
    };

    let document = match root.documents().next() {
      Some(document) => document,
      None => return Vec::new(),
    };

    let mut current_nodes = vec![document.syntax().clone()];

    if segments.is_empty() {
      if let Some(sequence) = document.syntax().descendants().find_map(BlockSeq::cast) {
        current_nodes = sequence
          .entries()
          .map(|entry| entry.syntax().clone())
          .collect();
      }

      return current_nodes;
    }

    for segment in &segments {
      let mut next_nodes = Vec::new();

      for node in &current_nodes {
        next_nodes.extend(resolve_segment(node, segment));
      }

      current_nodes = next_nodes;

      if current_nodes.is_empty() {
        break;
      }
    }

    current_nodes
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

  fn reorder_entries<T>(
    &mut self,
    entries: &[T],
    from: usize,
    to: usize,
    get_text: impl Fn(&T) -> String,
    get_indent: impl Fn(&T) -> String,
    range: TextRange,
  ) -> Result<(), YerbaError>
  where
    T: rowan::ast::AstNode,
  {
    let length = entries.len();

    if from >= length {
      return Err(YerbaError::IndexOutOfBounds(from, length));
    }

    if to >= length {
      return Err(YerbaError::IndexOutOfBounds(to, length));
    }

    let entry_texts: Vec<String> = entries.iter().map(&get_text).collect();

    let mut reordered = entry_texts.clone();
    let item = reordered.remove(from);

    reordered.insert(to, item);

    let indent = entries.get(1).map(&get_indent).unwrap_or_default();
    let text = rebuild_entries(reordered.iter().map(|text| text.as_str()), &indent);

    self.apply_edit(range, &text)
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

fn parse_condition(condition: &str) -> Option<(String, &str, String)> {
  let (left, operator, right) = if let Some(index) = condition.find(" not_contains ") {
    (
      condition[..index].trim(),
      "not_contains",
      condition[index + 14..].trim(),
    )
  } else if let Some(index) = condition.find(" contains ") {
    (
      condition[..index].trim(),
      "contains",
      condition[index + 10..].trim(),
    )
  } else if let Some(index) = condition.find("!=") {
    (
      condition[..index].trim(),
      "!=",
      condition[index + 2..].trim(),
    )
  } else if let Some(index) = condition.find("==") {
    (
      condition[..index].trim(),
      "==",
      condition[index + 2..].trim(),
    )
  } else {
    return None;
  };

  let right = right
    .trim_start_matches('"')
    .trim_end_matches('"')
    .trim_start_matches('\'')
    .trim_end_matches('\'');

  Some((left.to_string(), operator, right.to_string()))
}

fn parse_path_segments(path: &str) -> Vec<&str> {
  if path.is_empty() {
    return Vec::new();
  }

  let mut segments = Vec::new();
  let mut rest = path;

  while !rest.is_empty() {
    if rest.starts_with('[') {
      if let Some(close) = rest.find(']') {
        segments.push(&rest[..close + 1]);
        rest = &rest[close + 1..];

        if rest.starts_with('.') {
          rest = &rest[1..];
        }
      } else {
        segments.push(rest);
        break;
      }
    } else {
      let dot_index = rest.find('.');
      let bracket_index = rest.find('[');

      let split_at = match (dot_index, bracket_index) {
        (Some(dot), Some(bracket)) => Some(dot.min(bracket)),
        (Some(dot), None) => Some(dot),
        (None, Some(bracket)) => Some(bracket),
        (None, None) => None,
      };

      match split_at {
        Some(index) => {
          let segment = &rest[..index];

          if !segment.is_empty() {
            segments.push(segment);
          }

          rest = &rest[index..];

          if rest.starts_with('.') {
            rest = &rest[1..];
          }
        }

        None => {
          segments.push(rest);
          break;
        }
      }
    }
  }

  segments
}

fn parse_bracket_index(segment: &str) -> Option<usize> {
  if segment == "[]" {
    return None;
  }

  segment
    .strip_prefix('[')
    .and_then(|rest| rest.strip_suffix(']'))
    .and_then(|inner| inner.parse::<usize>().ok())
}

fn resolve_segment(node: &SyntaxNode, segment: &str) -> Vec<SyntaxNode> {
  if segment.starts_with('[') {
    if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
      match parse_bracket_index(segment) {
        None => sequence
          .entries()
          .map(|entry| entry.syntax().clone())
          .collect(),

        Some(index) => sequence
          .entries()
          .nth(index)
          .map(|entry| vec![entry.syntax().clone()])
          .unwrap_or_default(),
      }
    } else {
      Vec::new()
    }
  } else {
    if let Some(map) = node.descendants().find_map(BlockMap::cast) {
      if let Some(entry) = find_entry_by_key(&map, segment) {
        if let Some(value) = entry.value() {
          return vec![value.syntax().clone()];
        }
      }
    }

    Vec::new()
  }
}

fn navigate_from_node(node: &SyntaxNode, path: &str) -> Vec<SyntaxNode> {
  let segments = parse_path_segments(path);
  let mut current_nodes = vec![node.clone()];

  for segment in &segments {
    let mut next_nodes = Vec::new();

    for current in &current_nodes {
      next_nodes.extend(resolve_segment(current, segment));
    }

    current_nodes = next_nodes;

    if current_nodes.is_empty() {
      break;
    }
  }

  current_nodes
}

fn byte_offset_to_line(source: &str, offset: usize) -> usize {
  source[..offset.min(source.len())]
    .chars()
    .filter(|character| *character == '\n')
    .count()
    + 1
}

fn rebuild_entries<'a>(entries: impl Iterator<Item = &'a str>, indent: &str) -> String {
  entries
    .enumerate()
    .map(|(index, text)| {
      if index == 0 {
        text.to_string()
      } else {
        format!("\n{}{}", indent, text)
      }
    })
    .collect()
}
