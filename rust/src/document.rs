use std::fs;
use std::path::{Path, PathBuf};

use rowan::ast::AstNode;
use rowan::TextRange;

use yaml_parser::ast::{BlockMap, BlockSeq, Root};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

use crate::error::YerbaError;
use crate::QuoteStyle;

use crate::syntax::{
  extract_scalar, extract_scalar_text, find_entry_by_key, find_scalar_token, format_scalar_value, is_map_key,
  is_yaml_non_string, preceding_whitespace_indent, preceding_whitespace_token, removal_range, unescape_double_quoted,
  unescape_single_quoted, ScalarValue,
};

#[derive(Debug)]
pub struct FindResult {
  pub text: String,
  pub line: usize,
  pub end_line: usize,
}

#[derive(Debug, Clone)]
pub struct SortField {
  pub path: String,
  pub ascending: bool,
}

impl SortField {
  pub fn asc(path: &str) -> Self {
    SortField {
      path: path.to_string(),
      ascending: true,
    }
  }

  pub fn desc(path: &str) -> Self {
    SortField {
      path: path.to_string(),
      ascending: false,
    }
  }

  pub fn parse(input: &str) -> Self {
    if let Some((path, direction)) = input.rsplit_once(':') {
      match direction {
        "desc" | "descending" => SortField::desc(path),
        _ => SortField::asc(input),
      }
    } else {
      SortField::asc(input)
    }
  }

  pub fn parse_list(input: &str) -> Vec<Self> {
    input.split(',').map(|field| SortField::parse(field.trim())).collect()
  }
}

#[derive(Debug)]
pub enum InsertPosition {
  At(usize),
  Last,
  Before(String),
  After(String),
  BeforeCondition(String),
  AfterCondition(String),
  FromSortOrder(Vec<String>),
}

#[derive(Debug)]
pub struct Document {
  root: SyntaxNode,
  path: Option<PathBuf>,
}

impl Document {
  pub fn parse(source: &str) -> Result<Self, YerbaError> {
    let tree = yaml_parser::parse(source).map_err(|error| YerbaError::ParseError(format!("{}", error)))?;

    Ok(Document { root: tree, path: None })
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

    let current_node = self.navigate(dot_path).ok()?;

    extract_scalar_text(&current_node)
  }

  pub fn get_all(&self, dot_path: &str) -> Vec<String> {
    self
      .navigate_all(dot_path)
      .iter()
      .filter_map(extract_scalar_text)
      .collect()
  }

  pub fn get_typed(&self, dot_path: &str) -> Option<ScalarValue> {
    if crate::selector::Selector::parse(dot_path).has_wildcard() {
      return self.get_all_typed(dot_path).into_iter().next();
    }

    let current_node = self.navigate(dot_path).ok()?;

    if current_node
      .descendants()
      .any(|child| child.kind() == SyntaxKind::BLOCK_MAP || child.kind() == SyntaxKind::BLOCK_SEQ)
    {
      return None;
    }

    extract_scalar(&current_node)
  }

  pub fn get_all_typed(&self, dot_path: &str) -> Vec<ScalarValue> {
    self
      .navigate_all(dot_path)
      .iter()
      .filter(|node| {
        !node
          .descendants()
          .any(|child| child.kind() == SyntaxKind::BLOCK_MAP || child.kind() == SyntaxKind::BLOCK_SEQ)
      })
      .filter_map(extract_scalar)
      .collect()
  }

  pub fn get_value(&self, dot_path: &str) -> Option<serde_yaml::Value> {
    if dot_path.is_empty() {
      return Some(node_to_yaml_value(&self.root));
    }

    let nodes = self.navigate_all(dot_path);

    if nodes.is_empty() {
      return None;
    }

    if nodes.len() == 1 {
      return Some(node_to_yaml_value(&nodes[0]));
    }

    let values: Vec<serde_yaml::Value> = nodes.iter().map(node_to_yaml_value).collect();

    Some(serde_yaml::Value::Sequence(values))
  }

  pub fn get_values(&self, dot_path: &str) -> Vec<serde_yaml::Value> {
    self.navigate_all(dot_path).iter().map(node_to_yaml_value).collect()
  }

  pub fn filter_values(&self, dot_path: &str, condition: &str) -> Vec<serde_yaml::Value> {
    self
      .navigate_all(dot_path)
      .iter()
      .filter(|node| self.evaluate_condition_on_node(node, condition))
      .map(node_to_yaml_value)
      .collect()
  }

  pub fn find_items(&self, dot_path: &str, condition: &str) -> Vec<FindResult> {
    let source = self.root.text().to_string();
    let nodes = self.navigate_all(dot_path);

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
      .navigate_all(dot_path)
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

    let path = crate::selector::Selector::parse(&left);

    if !path.is_relative() {
      return false;
    }

    let target_nodes = navigate_from_node(node, &path.to_selector_string());
    let values: Vec<String> = target_nodes.iter().filter_map(extract_scalar_text).collect();

    match operator {
      "==" => values.iter().any(|value| value == &right),
      "!=" => values.iter().all(|value| value != &right),
      "contains" => {
        if values.iter().any(|value| value == &right || value.contains(&right)) {
          return true;
        }

        for node in &target_nodes {
          if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
            for entry in sequence.entries() {
              if let Some(text) = entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())) {
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
              if let Some(text) = entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())) {
                if text == right {
                  return false;
                }
              }
            }
          }
        }

        !values.iter().any(|value| value == &right || value.contains(&right))
      }
      _ => false,
    }
  }

  pub fn exists(&self, dot_path: &str) -> bool {
    if dot_path.contains('[') {
      return !self.navigate_all(dot_path).is_empty();
    }

    self.get(dot_path).is_some()
  }

  pub fn evaluate_condition(&self, parent_path: &str, condition: &str) -> bool {
    let condition = condition.trim();

    let (left, operator, right) = match parse_condition(condition) {
      Some(parts) => parts,
      None => return false,
    };

    let path = crate::selector::Selector::parse(&left);

    let full_path = if path.is_relative() {
      let path_string = path.to_selector_string();

      if parent_path.is_empty() {
        path_string
      } else {
        format!("{}.{}", parent_path, path_string)
      }
    } else {
      path.to_selector_string()
    };

    let has_brackets = crate::selector::Selector::parse(&full_path).has_brackets();

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
    let current_node = match self.navigate(dot_path) {
      Ok(node) => node,
      Err(_) => return Vec::new(),
    };

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Vec::new(),
    };

    sequence
      .entries()
      .filter_map(|entry| entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())))
      .collect()
  }

  pub fn set(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    if let Some(block_scalar) = current_node
      .descendants()
      .find(|node| node.kind() == SyntaxKind::BLOCK_SCALAR)
    {
      let new_text = if value.is_empty() {
        "\"\"".to_string()
      } else if value.contains('\n') {
        format!("|-\n  {}", value.replace('\n', "\n  "))
      } else {
        format!("\"{}\"", value.replace('"', "\\\""))
      };

      let range = block_scalar.text_range();

      return self.apply_edit(range, &new_text);
    }

    let scalar_token =
      find_scalar_token(&current_node).ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let new_text = format_scalar_value(value, scalar_token.kind());

    self.replace_token(&scalar_token, &new_text)
  }

  pub fn set_scalar_style(&mut self, dot_path: &str, style: &QuoteStyle) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;
    let scalar_token =
      find_scalar_token(&current_node).ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let current_kind = scalar_token.kind();
    let target_kind = style.to_syntax_kind();

    if current_kind == target_kind {
      return Ok(());
    }

    let raw_value = match current_kind {
      SyntaxKind::DOUBLE_QUOTED_SCALAR => {
        let text = scalar_token.text();
        unescape_double_quoted(&text[1..text.len() - 1])
      }

      SyntaxKind::SINGLE_QUOTED_SCALAR => {
        let text = scalar_token.text();
        unescape_single_quoted(&text[1..text.len() - 1])
      }

      SyntaxKind::PLAIN_SCALAR => scalar_token.text().to_string(),

      _ => return Ok(()),
    };

    let new_text = format_scalar_value(&raw_value, target_kind);

    self.replace_token(&scalar_token, &new_text)
  }

  pub fn set_plain(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    if let Some(block_scalar) = current_node
      .descendants()
      .find(|node| node.kind() == SyntaxKind::BLOCK_SCALAR)
    {
      let range = block_scalar.text_range();
      return self.apply_edit(range, value);
    }

    let scalar_token =
      find_scalar_token(&current_node).ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    self.replace_token(&scalar_token, value)
  }

  pub fn append(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    self.insert_into(dot_path, value, InsertPosition::Last)
  }

  pub fn insert_into(&mut self, dot_path: &str, value: &str, position: InsertPosition) -> Result<(), YerbaError> {
    Self::validate_path(dot_path)?;

    if let Ok(current_node) = self.navigate(dot_path) {
      if current_node.descendants().find_map(BlockSeq::cast).is_some() {
        return self.insert_sequence_item(dot_path, value, position);
      }
    }

    let (parent_path, key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));

    self.insert_map_key(parent_path, key, value, position)
  }

  fn insert_sequence_item(&mut self, dot_path: &str, value: &str, position: InsertPosition) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

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

    let new_item = if value.contains('\n') {
      let item_indent = format!("{}  ", indent);
      let lines: Vec<&str> = value.split('\n').collect();

      let min_indent = lines
        .iter()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

      let indented: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
          if index == 0 {
            line.to_string()
          } else if line.trim().is_empty() {
            String::new()
          } else {
            let relative = &line[min_indent..];
            format!("{}{}", item_indent, relative)
          }
        })
        .collect();

      format!("- {}", indented.join("\n"))
    } else {
      format!("- {}", value)
    };

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
          .ok_or_else(|| YerbaError::PathNotFound(format!("{} item '{}'", dot_path, target_value)))?;

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
          .ok_or_else(|| YerbaError::PathNotFound(format!("{} item '{}'", dot_path, target_value)))?;

        let new_text = format!("\n{}{}", indent, new_item);

        self.insert_after_node(target_entry.syntax(), &new_text)
      }

      InsertPosition::BeforeCondition(condition) => {
        let target_entry = entries
          .iter()
          .find(|entry| self.evaluate_condition_on_node(entry.syntax(), &condition))
          .ok_or_else(|| YerbaError::PathNotFound(format!("{} condition '{}'", dot_path, condition)))?;

        let target_range = target_entry.syntax().text_range();
        let replacement = format!("{}\n{}", new_item, indent);
        let insert_range = TextRange::new(target_range.start(), target_range.start());

        self.apply_edit(insert_range, &replacement)
      }

      InsertPosition::AfterCondition(condition) => {
        let target_entry = entries
          .iter()
          .find(|entry| self.evaluate_condition_on_node(entry.syntax(), &condition))
          .ok_or_else(|| YerbaError::PathNotFound(format!("{} condition '{}'", dot_path, condition)))?;

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
    let current_node = self.navigate(dot_path)?;

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

      InsertPosition::BeforeCondition(_) | InsertPosition::AfterCondition(_) => {
        self.insert_map_key(dot_path, key, value, InsertPosition::Last)
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
    Self::validate_path(source_path)?;
    Self::validate_path(destination_path)?;

    let source_parent = source_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

    let destination_parent = destination_path
      .rsplit_once('.')
      .map(|(parent, _)| parent)
      .unwrap_or("");

    let destination_key = destination_path
      .rsplit_once('.')
      .map(|(_, key)| key)
      .unwrap_or(destination_path);

    if source_parent == destination_parent {
      let (parent_path, source_key) = source_path.rsplit_once('.').unwrap_or(("", source_path));
      let parent_node = self.navigate(parent_path)?;

      let map = parent_node
        .descendants()
        .find_map(BlockMap::cast)
        .ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      let entry =
        find_entry_by_key(&map, source_key).ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      let key_node = entry
        .key()
        .ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

      let key_token =
        find_scalar_token(key_node.syntax()).ok_or_else(|| YerbaError::PathNotFound(source_path.to_string()))?;

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
    Self::validate_path(dot_path)?;

    let (parent_path, last_key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));
    let parent_node = self.navigate(parent_path)?;

    let map = parent_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entry = find_entry_by_key(&map, last_key).ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    self.remove_node(entry.syntax())
  }

  pub fn remove(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

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

  pub fn remove_at(&mut self, dot_path: &str, index: usize) -> Result<(), YerbaError> {
    Self::validate_path(dot_path)?;

    let current_node = self.navigate(dot_path)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    if index >= entries.len() {
      return Err(YerbaError::IndexOutOfBounds(index, entries.len()));
    }

    self.remove_node(entries[index].syntax())
  }

  pub fn move_item(&mut self, dot_path: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let current_node = self.navigate(dot_path)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    self.reorder_entries(sequence.syntax(), &entries, from, to)
  }

  pub fn move_key(&mut self, dot_path: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let current_node = self.navigate(dot_path)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entries: Vec<_> = map.entries().collect();

    self.reorder_entries(map.syntax(), &entries, from, to)
  }

  pub fn resolve_key_index(&self, dot_path: &str, reference: &str) -> Result<usize, YerbaError> {
    let current_node = self.navigate(dot_path)?;

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

  pub fn resolve_sequence_index(&self, dot_path: &str, reference: &str) -> Result<usize, YerbaError> {
    let current_node = self.navigate(dot_path)?;

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

    if crate::selector::Selector::parse(reference).is_relative() {
      return sequence
        .entries()
        .enumerate()
        .find(|(_index, entry)| self.evaluate_condition_on_node(entry.syntax(), reference))
        .map(|(index, _entry)| index)
        .ok_or_else(|| YerbaError::PathNotFound(format!("{} condition '{}'", dot_path, reference)));
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

    let current_node = self.navigate(dot_path)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let unknown_keys: Vec<String> = map
      .entries()
      .filter_map(|entry| entry.key().and_then(|key_node| extract_scalar_text(key_node.syntax())))
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

    let current_node = self.navigate(dot_path)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

    let entries: Vec<_> = map.entries().collect();

    if entries.len() <= 1 {
      return Ok(());
    }

    let (groups, range) = collect_groups_with_range(map.syntax());

    let mut keyed: Vec<(String, EntryGroup)> = entries
      .iter()
      .zip(groups)
      .map(|(entry, group)| {
        let key_name = entry
          .key()
          .and_then(|key_node| extract_scalar_text(key_node.syntax()))
          .unwrap_or_default();
        (key_name, group)
      })
      .collect();

    let original_keys: Vec<String> = keyed.iter().map(|(key, _)| key.clone()).collect();

    keyed.sort_by(|(key_a, _), (key_b, _)| {
      let position_a = key_order.iter().position(|&key| key == key_a);
      let position_b = key_order.iter().position(|&key| key == key_b);

      match (position_a, position_b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => {
          let original_a = original_keys.iter().position(|key| key == key_a).unwrap();
          let original_b = original_keys.iter().position(|key| key == key_b).unwrap();
          original_a.cmp(&original_b)
        }
      }
    });

    let sorted_keys: Vec<&str> = keyed.iter().map(|(key, _)| key.as_str()).collect();
    let orig_refs: Vec<&str> = original_keys.iter().map(|key| key.as_str()).collect();

    if sorted_keys == orig_refs {
      return Ok(());
    }

    let indent = entries
      .get(1)
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let sorted_groups: Vec<EntryGroup> = keyed.into_iter().map(|(_, group)| group).collect();
    let map_text = rebuild_from_groups(&sorted_groups, &indent);

    self.apply_edit(range, &map_text)
  }

  pub fn sort_each_keys(&mut self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

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

      let (groups, group_range) = collect_groups_with_range(map.syntax());

      let mut keyed: Vec<(String, EntryGroup)> = entries
        .iter()
        .zip(groups)
        .map(|(entry, group)| {
          let key_name = entry
            .key()
            .and_then(|key_node| extract_scalar_text(key_node.syntax()))
            .unwrap_or_default();
          (key_name, group)
        })
        .collect();

      let original_keys: Vec<String> = keyed.iter().map(|(key, _)| key.clone()).collect();

      keyed.sort_by(|(key_a, _), (key_b, _)| {
        let position_a = key_order.iter().position(|&key| key == key_a);
        let position_b = key_order.iter().position(|&key| key == key_b);

        match (position_a, position_b) {
          (Some(a), Some(b)) => a.cmp(&b),
          (Some(_), None) => std::cmp::Ordering::Less,
          (None, Some(_)) => std::cmp::Ordering::Greater,

          (None, None) => {
            let original_a = original_keys.iter().position(|key| key == key_a).unwrap();
            let original_b = original_keys.iter().position(|key| key == key_b).unwrap();

            original_a.cmp(&original_b)
          }
        }
      });

      let sorted_keys: Vec<&str> = keyed.iter().map(|(key, _)| key.as_str()).collect();
      let orig_refs: Vec<&str> = original_keys.iter().map(|key| key.as_str()).collect();

      if sorted_keys == orig_refs {
        continue;
      }

      let indent = entries
        .get(1)
        .map(|entry| preceding_whitespace_indent(entry.syntax()))
        .unwrap_or_default();

      let sorted_groups: Vec<EntryGroup> = keyed.into_iter().map(|(_, group)| group).collect();
      let map_text = rebuild_from_groups(&sorted_groups, &indent);
      edits.push((group_range, map_text));
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

  pub fn validate_each_sort_keys(&self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

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

  pub fn sort_items(
    &mut self,
    dot_path: &str,
    sort_fields: &[SortField],
    case_sensitive: bool,
  ) -> Result<(), YerbaError> {
    if dot_path.contains("[].") {
      return self.sort_each_items(dot_path, sort_fields, case_sensitive);
    }

    let current_node = self.navigate(dot_path)?;

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    let entries: Vec<_> = sequence.entries().collect();

    if entries.len() <= 1 {
      return Ok(());
    }

    let (groups, range) = collect_groups_with_range(sequence.syntax());

    let mut sortable: Vec<(Vec<String>, EntryGroup)> = entries
      .iter()
      .zip(groups)
      .map(|(entry, group)| {
        let sort_values = if sort_fields.is_empty() {
          vec![entry
            .flow()
            .and_then(|flow| extract_scalar_text(flow.syntax()))
            .unwrap_or_default()]
        } else {
          sort_fields
            .iter()
            .map(|field| {
              let nodes = navigate_from_node(entry.syntax(), &field.path);
              nodes.first().and_then(extract_scalar_text).unwrap_or_default()
            })
            .collect()
        };

        (sort_values, group)
      })
      .collect();

    let original_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

    sortable.sort_by(|(values_a, _), (values_b, _)| {
      for (index, field) in sort_fields.iter().enumerate().take(values_a.len()) {
        let value_a = &values_a[index];
        let value_b = &values_b[index];

        let ordering = if case_sensitive {
          value_a.cmp(value_b)
        } else {
          value_a.to_lowercase().cmp(&value_b.to_lowercase())
        };

        let ordering = if field.ascending { ordering } else { ordering.reverse() };

        if ordering != std::cmp::Ordering::Equal {
          return ordering;
        }
      }

      if sort_fields.is_empty() && !values_a.is_empty() && !values_b.is_empty() {
        return if case_sensitive {
          values_a[0].cmp(&values_b[0])
        } else {
          values_a[0].to_lowercase().cmp(&values_b[0].to_lowercase())
        };
      }

      std::cmp::Ordering::Equal
    });

    let sorted_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

    if sorted_bodies == original_bodies {
      return Ok(());
    }

    let indent = entries
      .get(1)
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let sorted_groups: Vec<EntryGroup> = sortable.into_iter().map(|(_, group)| group).collect();
    let sequence_text = rebuild_from_groups(&sorted_groups, &indent);

    self.apply_edit(range, &sequence_text)
  }

  fn sort_each_items(
    &mut self,
    dot_path: &str,
    sort_fields: &[SortField],
    case_sensitive: bool,
  ) -> Result<(), YerbaError> {
    let (parent_path, child_path) = if let Some(last_bracket) = dot_path.rfind("[].") {
      (&dot_path[..last_bracket + 2], &dot_path[last_bracket + 3..])
    } else {
      (dot_path, "")
    };

    let parent_nodes = self.navigate_all(parent_path);
    let source = self.root.text().to_string();
    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for parent_node in &parent_nodes {
      let child_nodes = if child_path.is_empty() {
        vec![parent_node.clone()]
      } else {
        navigate_from_node(parent_node, child_path)
      };

      for child_node in &child_nodes {
        let sequence = match child_node.descendants().find_map(BlockSeq::cast) {
          Some(sequence) => sequence,
          None => continue,
        };

        let entries: Vec<_> = sequence.entries().collect();

        if entries.len() <= 1 {
          continue;
        }

        let (groups, group_range) = collect_groups_with_range(sequence.syntax());

        let mut sortable: Vec<(Vec<String>, EntryGroup)> = entries
          .iter()
          .zip(groups)
          .map(|(entry, group)| {
            let sort_values = if sort_fields.is_empty() {
              vec![entry
                .flow()
                .and_then(|flow| extract_scalar_text(flow.syntax()))
                .unwrap_or_default()]
            } else {
              sort_fields
                .iter()
                .map(|field| {
                  let nodes = navigate_from_node(entry.syntax(), &field.path);
                  nodes.first().and_then(extract_scalar_text).unwrap_or_default()
                })
                .collect()
            };

            (sort_values, group)
          })
          .collect();

        let original_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

        sortable.sort_by(|(values_a, _), (values_b, _)| {
          for (index, field) in sort_fields.iter().enumerate().take(values_a.len()) {
            let value_a = &values_a[index];
            let value_b = &values_b[index];

            let ordering = if case_sensitive {
              value_a.cmp(value_b)
            } else {
              value_a.to_lowercase().cmp(&value_b.to_lowercase())
            };

            let ordering = if field.ascending { ordering } else { ordering.reverse() };

            if ordering != std::cmp::Ordering::Equal {
              return ordering;
            }
          }

          if sort_fields.is_empty() && !values_a.is_empty() && !values_b.is_empty() {
            return if case_sensitive {
              values_a[0].cmp(&values_b[0])
            } else {
              values_a[0].to_lowercase().cmp(&values_b[0].to_lowercase())
            };
          }

          std::cmp::Ordering::Equal
        });

        let sorted_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

        if sorted_bodies == original_bodies {
          continue;
        }

        let indent = entries
          .get(1)
          .map(|entry| preceding_whitespace_indent(entry.syntax()))
          .unwrap_or_default();

        let sorted_groups: Vec<EntryGroup> = sortable.into_iter().map(|(_, group)| group).collect();
        let sequence_text = rebuild_from_groups(&sorted_groups, &indent);
        edits.push((group_range, sequence_text));
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

  pub fn enforce_blank_lines(&mut self, dot_path: &str, blank_lines: usize) -> Result<(), YerbaError> {
    let nodes = if dot_path.contains('[') {
      self.navigate_all(dot_path)
    } else {
      vec![self.navigate(dot_path)?]
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for current_node in &nodes {
      let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
        Some(sequence) => sequence,
        None => continue,
      };

      let entries: Vec<_> = sequence.entries().collect();

      if entries.len() <= 1 {
        continue;
      }

      for entry in entries.iter().skip(1) {
        if let Some(whitespace_token) = preceding_whitespace_token(entry.syntax()) {
          let whitespace_text = whitespace_token.text();
          let newline_count = whitespace_text.chars().filter(|character| *character == '\n').count();

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
    }

    if edits.is_empty() {
      return Ok(());
    }

    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0.start()));

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

  pub fn enforce_key_style(&mut self, style: &QuoteStyle, dot_path: Option<&str>) -> Result<(), YerbaError> {
    let source = self.root.text().to_string();

    let scope_ranges: Vec<TextRange> = match dot_path {
      Some(path) if !path.is_empty() => self.navigate_all(path).iter().map(|node| node.text_range()).collect(),
      _ => vec![self.root.text_range()],
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for element in self.root.descendants_with_tokens() {
      if let Some(token) = element.into_token() {
        if !scope_ranges
          .iter()
          .any(|range| range.contains_range(token.text_range()))
        {
          continue;
        }

        if !is_map_key(&token) {
          continue;
        }

        let current_kind = token.kind();

        if !matches!(
          current_kind,
          SyntaxKind::PLAIN_SCALAR | SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR
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

  pub fn enforce_quotes_at(&mut self, style: &QuoteStyle, dot_path: Option<&str>) -> Result<(), YerbaError> {
    let source = self.root.text().to_string();

    let scope_ranges: Vec<TextRange> = match dot_path {
      Some(path) if !path.is_empty() => self.navigate_all(path).iter().map(|node| node.text_range()).collect(),
      _ => vec![self.root.text_range()],
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for element in self.root.descendants_with_tokens() {
      if let Some(token) = element.into_token() {
        if !scope_ranges
          .iter()
          .any(|range| range.contains_range(token.text_range()))
        {
          continue;
        }

        if is_map_key(&token) {
          continue;
        }

        let current_kind = token.kind();

        if !matches!(
          current_kind,
          SyntaxKind::PLAIN_SCALAR | SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR
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
            if raw_value.contains('"') || raw_value.contains('\'') || raw_value.contains(':') || raw_value.contains('#')
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

  pub fn navigate(&self, dot_path: &str) -> Result<SyntaxNode, YerbaError> {
    Self::validate_path(dot_path)?;

    if dot_path.is_empty() {
      let root = Root::cast(self.root.clone()).ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

      let document = root
        .documents()
        .next()
        .ok_or_else(|| YerbaError::PathNotFound(dot_path.to_string()))?;

      return Ok(document.syntax().clone());
    }

    let nodes = self.navigate_all(dot_path);

    match nodes.len() {
      0 => Err(YerbaError::PathNotFound(dot_path.to_string())),
      1 => Ok(nodes.into_iter().next().unwrap()),
      _ => Err(YerbaError::PathNotFound(format!(
        "{} (matched {} nodes, expected 1)",
        dot_path,
        nodes.len()
      ))),
    }
  }

  pub fn validate_path(dot_path: &str) -> Result<(), YerbaError> {
    if dot_path.ends_with('.') {
      return Err(YerbaError::ParseError(format!(
        "invalid path: trailing dot in '{}'",
        dot_path
      )));
    }

    if dot_path.contains("..") {
      return Err(YerbaError::ParseError(format!(
        "invalid path: double dot in '{}'",
        dot_path
      )));
    }

    if dot_path.starts_with('.') {
      return Err(YerbaError::ParseError(format!(
        "invalid path: leading dot in '{}'",
        dot_path
      )));
    }

    if dot_path.contains('[') && !dot_path.contains(']') {
      return Err(YerbaError::ParseError(format!(
        "invalid path: unclosed bracket in '{}'",
        dot_path
      )));
    }

    Ok(())
  }

  pub fn navigate_all(&self, dot_path: &str) -> Vec<SyntaxNode> {
    if Document::validate_path(dot_path).is_err() {
      return Vec::new();
    }

    let parsed = crate::selector::Selector::parse(dot_path);

    let root = match Root::cast(self.root.clone()) {
      Some(root) => root,
      None => return Vec::new(),
    };

    let document = match root.documents().next() {
      Some(document) => document,
      None => return Vec::new(),
    };

    let mut current_nodes = vec![document.syntax().clone()];

    if parsed.is_empty() {
      if let Some(sequence) = document.syntax().descendants().find_map(BlockSeq::cast) {
        current_nodes = sequence.entries().map(|entry| entry.syntax().clone()).collect();
      }

      return current_nodes;
    }

    for segment in parsed.segments() {
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
    let inline_comment = self.find_inline_comment(node);
    let range = removal_range(node);

    if let Some((comment_text, comment_end)) = inline_comment {
      let indent = preceding_whitespace_indent(node);
      let replacement = format!("\n{}{}", indent, comment_text);
      let expanded_range = TextRange::new(range.start(), comment_end);

      self.apply_edit(expanded_range, &replacement)
    } else {
      self.apply_edit(range, "")
    }
  }

  fn find_inline_comment(&self, node: &SyntaxNode) -> Option<(String, rowan::TextSize)> {
    let mut sibling = node.next_sibling_or_token();

    while let Some(ref element) = sibling {
      match element {
        rowan::NodeOrToken::Token(token) => {
          if token.kind() == SyntaxKind::COMMENT {
            return Some((token.text().to_string(), token.text_range().end()));
          } else if token.kind() == SyntaxKind::WHITESPACE {
            if token.text().contains('\n') {
              return None;
            }
          } else {
            return None;
          }
        }
        _ => return None,
      }

      sibling = match element {
        rowan::NodeOrToken::Token(token) => token.next_sibling_or_token(),
        rowan::NodeOrToken::Node(node) => node.next_sibling_or_token(),
      };
    }

    None
  }

  fn reorder_entries<T>(&mut self, parent: &SyntaxNode, entries: &[T], from: usize, to: usize) -> Result<(), YerbaError>
  where
    T: rowan::ast::AstNode<Language = yaml_parser::YamlLanguage>,
  {
    let length = entries.len();

    if from >= length {
      return Err(YerbaError::IndexOutOfBounds(from, length));
    }

    if to >= length {
      return Err(YerbaError::IndexOutOfBounds(to, length));
    }

    let (groups, range) = collect_groups_with_range(parent);

    let mut reordered = groups.clone();
    let item = reordered.remove(from);
    reordered.insert(to, item);

    let indent = entries
      .get(1)
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let text = rebuild_from_groups(&reordered, &indent);

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

pub fn node_to_yaml_value(node: &SyntaxNode) -> serde_yaml::Value {
  if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
    let map_position = node
      .descendants()
      .find_map(BlockMap::cast)
      .map(|map| map.syntax().text_range().start());

    let sequence_position = sequence.syntax().text_range().start();

    if map_position.is_none() || sequence_position <= map_position.unwrap() {
      let values: Vec<serde_yaml::Value> = sequence
        .entries()
        .map(|entry| node_to_yaml_value(entry.syntax()))
        .collect();

      return serde_yaml::Value::Sequence(values);
    }
  }

  if let Some(map) = node.descendants().find_map(BlockMap::cast) {
    let mut mapping = serde_yaml::Mapping::new();

    for entry in map.entries() {
      let key = entry
        .key()
        .and_then(|key_node| extract_scalar_text(key_node.syntax()))
        .unwrap_or_default();

      let value = entry
        .value()
        .map(|value_node| node_to_yaml_value(value_node.syntax()))
        .unwrap_or(serde_yaml::Value::Null);

      mapping.insert(serde_yaml::Value::String(key), value);
    }

    return serde_yaml::Value::Mapping(mapping);
  }

  if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
    let values: Vec<serde_yaml::Value> = sequence
      .entries()
      .map(|entry| node_to_yaml_value(entry.syntax()))
      .collect();

    return serde_yaml::Value::Sequence(values);
  }

  if let Some(block_scalar) = node
    .descendants()
    .find(|child| child.kind() == SyntaxKind::BLOCK_SCALAR)
  {
    let text = block_scalar
      .descendants_with_tokens()
      .filter_map(|element| element.into_token())
      .find(|token| token.kind() == SyntaxKind::BLOCK_SCALAR_TEXT)
      .map(|token| token.text().to_string())
      .unwrap_or_default();

    return serde_yaml::Value::String(text);
  }

  if let Some(scalar) = extract_scalar(node) {
    use crate::syntax::{detect_yaml_type, is_yaml_truthy, YerbaValueType};

    return match detect_yaml_type(&scalar) {
      YerbaValueType::Null => serde_yaml::Value::Null,
      YerbaValueType::Boolean => serde_yaml::Value::Bool(is_yaml_truthy(&scalar.text)),

      YerbaValueType::Integer => scalar
        .text
        .parse::<i64>()
        .map(|n| serde_yaml::Value::Number(serde_yaml::Number::from(n)))
        .unwrap_or(serde_yaml::Value::String(scalar.text)),

      YerbaValueType::Float => scalar
        .text
        .parse::<f64>()
        .map(|n| serde_yaml::Value::Number(serde_yaml::Number::from(n)))
        .unwrap_or(serde_yaml::Value::String(scalar.text)),

      YerbaValueType::String => serde_yaml::Value::String(scalar.text),
    };
  }

  let text = node.text().to_string();

  serde_yaml::from_str(&text).unwrap_or(serde_yaml::Value::String(text))
}

fn parse_condition(condition: &str) -> Option<(String, &str, String)> {
  let (left, operator, right) = if let Some(index) = condition.find(" not_contains ") {
    (
      condition[..index].trim(),
      "not_contains",
      condition[index + 14..].trim(),
    )
  } else if let Some(index) = condition.find(" contains ") {
    (condition[..index].trim(), "contains", condition[index + 10..].trim())
  } else if let Some(index) = condition.find("!=") {
    (condition[..index].trim(), "!=", condition[index + 2..].trim())
  } else if let Some(index) = condition.find("==") {
    (condition[..index].trim(), "==", condition[index + 2..].trim())
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

fn resolve_segment(node: &SyntaxNode, segment: &crate::selector::SelectorSegment) -> Vec<SyntaxNode> {
  use crate::selector::SelectorSegment;

  match segment {
    SelectorSegment::AllItems => {
      if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
        sequence.entries().map(|entry| entry.syntax().clone()).collect()
      } else {
        Vec::new()
      }
    }

    SelectorSegment::Index(index) => {
      if let Some(sequence) = node.descendants().find_map(BlockSeq::cast) {
        sequence
          .entries()
          .nth(*index)
          .map(|entry| vec![entry.syntax().clone()])
          .unwrap_or_default()
      } else {
        Vec::new()
      }
    }

    SelectorSegment::Key(key) => {
      if let Some(map) = node.descendants().find_map(BlockMap::cast) {
        if let Some(entry) = find_entry_by_key(&map, key) {
          if let Some(value) = entry.value() {
            return vec![value.syntax().clone()];
          }
        }
      }

      Vec::new()
    }
  }
}

fn navigate_from_node(node: &SyntaxNode, path: &str) -> Vec<SyntaxNode> {
  let parsed = crate::selector::Selector::parse(path);
  let mut current_nodes = vec![node.clone()];

  for segment in parsed.segments() {
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

#[derive(Debug, Clone)]
struct EntryGroup {
  separator: String,
  preceding: String,
  body: String,
}

impl EntryGroup {
  fn full_text(&self) -> String {
    if self.preceding.is_empty() {
      self.body.clone()
    } else {
      format!("{}\n{}", self.preceding, self.body)
    }
  }
}

fn collect_entry_groups(parent: &SyntaxNode) -> Vec<EntryGroup> {
  let mut groups: Vec<EntryGroup> = Vec::new();
  let mut buffer = String::new();

  for child in parent.children_with_tokens() {
    let is_entry = child.as_node().is_some()
      && matches!(
        child.as_node().unwrap().kind(),
        SyntaxKind::BLOCK_MAP_ENTRY | SyntaxKind::BLOCK_SEQ_ENTRY
      );

    if is_entry {
      let entry_text = child.as_node().unwrap().text().to_string();

      if groups.is_empty() {
        let preceding = buffer.trim_start_matches('\n').to_string();

        groups.push(EntryGroup {
          separator: String::new(),
          preceding,
          body: entry_text,
        });
      } else {
        let (trailing, separator, preceding) = split_at_blank_line(&buffer);

        if let Some(last) = groups.last_mut() {
          last.body.push_str(&trailing);
        }

        groups.push(EntryGroup {
          separator,
          preceding,
          body: entry_text,
        });
      }

      buffer.clear();
    } else {
      let text = match &child {
        rowan::NodeOrToken::Node(node) => node.text().to_string(),
        rowan::NodeOrToken::Token(token) => token.text().to_string(),
      };

      buffer.push_str(&text);
    }
  }

  if let Some(last) = groups.last_mut() {
    let trimmed = buffer.trim_end_matches(['\n', ' ', '\t']);

    if !trimmed.is_empty() {
      last.body.push_str(trimmed);
    }
  }

  for group in &mut groups {
    let trimmed = group.body.trim_end_matches(['\n', ' ', '\t']);

    group.body = trimmed.to_string();
  }

  groups
}

fn split_at_blank_line(text: &str) -> (String, String, String) {
  if let Some(position) = text.find("\n\n") {
    let trailing = text[..position].to_string();
    let rest = &text[position..];
    let content_start = rest.len() - rest.trim_start_matches('\n').len();
    let separator = rest[..content_start].to_string();
    let preceding = rest[content_start..].trim_end_matches(['\n', ' ', '\t']).to_string();

    (trailing, separator, preceding)
  } else {
    (text.to_string(), String::new(), String::new())
  }
}

fn collect_preceding_sibling_comments(parent: &SyntaxNode) -> (String, Option<rowan::TextSize>) {
  let mut comments: Vec<String> = Vec::new();
  let mut earliest_start = None;
  let mut node = parent.clone();

  loop {
    let mut sibling = node.prev_sibling_or_token();

    while let Some(ref element) = sibling {
      match element {
        rowan::NodeOrToken::Token(token) => {
          if token.kind() == SyntaxKind::COMMENT {
            comments.push(token.text().to_string());
            earliest_start = Some(token.text_range().start());
          } else if token.kind() == SyntaxKind::WHITESPACE {
            // Keep looking past whitespace
          } else {
            break;
          }
        }
        _ => break,
      }

      sibling = match element {
        rowan::NodeOrToken::Token(token) => token.prev_sibling_or_token(),
        rowan::NodeOrToken::Node(node) => node.prev_sibling_or_token(),
      };
    }

    if !comments.is_empty() {
      break;
    }

    match node.parent() {
      Some(parent)
        if parent.kind() == SyntaxKind::BLOCK
          || parent.kind() == SyntaxKind::DOCUMENT
          || parent.kind() == SyntaxKind::BLOCK_MAP_VALUE
          || parent.kind() == SyntaxKind::BLOCK_SEQ_ENTRY =>
      {
        node = parent
      }
      _ => break,
    }
  }

  comments.reverse();
  (comments.join("\n"), earliest_start)
}

fn collect_groups_with_range(parent: &SyntaxNode) -> (Vec<EntryGroup>, TextRange) {
  let mut groups = collect_entry_groups(parent);

  let (sibling_comments, earliest_start) = collect_preceding_sibling_comments(parent);

  if !sibling_comments.is_empty() {
    if let Some(first) = groups.first_mut() {
      if first.preceding.is_empty() {
        first.preceding = sibling_comments;
      } else {
        first.preceding = format!("{}\n{}", sibling_comments, first.preceding);
      }
    }
  }

  let range = match earliest_start {
    Some(start) => TextRange::new(start, parent.text_range().end()),
    None => parent.text_range(),
  };

  (groups, range)
}

fn rebuild_from_groups(groups: &[EntryGroup], indent: &str) -> String {
  let default_separator = groups
    .iter()
    .find(|group| !group.separator.is_empty())
    .map(|group| group.separator.clone())
    .unwrap_or_else(|| "\n".to_string());

  groups
    .iter()
    .enumerate()
    .map(|(index, group)| {
      if index == 0 {
        group.full_text()
      } else if group.preceding.is_empty() {
        format!("{}{}{}", default_separator, indent, group.body)
      } else {
        format!("{}{}\n{}{}", default_separator, group.preceding, indent, group.body)
      }
    })
    .collect()
}
