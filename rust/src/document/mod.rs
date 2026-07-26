mod condition;
pub use condition::{validate_condition, validate_item_condition};
mod delete;
mod get;
mod insert;
mod schema;
mod set;
mod sort;
pub mod style;
mod unique;
pub use unique::DuplicateInfo;

use crate::syntax::YerbaValueType;

#[derive(Debug, Clone)]
pub struct LocatedNode {
  pub node_type: String,
  pub text: Option<String>,
  pub value_type: Option<YerbaValueType>,
  pub file_path: Option<String>,
  pub selector: String,
  pub line: usize,
  pub location: Location,
  pub key_name: Option<String>,
  pub key_location: Location,
}

use std::fs;
use std::path::{Path, PathBuf};

use rowan::ast::AstNode;
use rowan::{TextRange, TextSize};

use yaml_parser::ast::{BlockMap, BlockMapEntry, BlockSeq, Root};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

use crate::error::YerbaError;
use crate::QuoteStyle;

use crate::syntax::{
  column_at, dedent_block_scalar, extract_scalar, extract_scalar_text, find_block_map, find_block_sequence, find_entry_by_key, find_flow_entry_by_key,
  find_flow_map, find_flow_sequence, find_scalar_token, first_collection, flow_map_entries, flow_sequence_entries, format_scalar_value, in_flow_collection,
  is_map_key, is_yaml_non_string, line_at, line_start_at, preceding_whitespace_indent, preceding_whitespace_token, raw_scalar_value, removal_range,
  FirstCollection, ScalarValue,
};

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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
  Scalar = 0,
  Map = 1,
  Sequence = 2,
  NotFound = 3,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Location {
  pub start_line: usize,
  pub start_column: usize,
  pub end_line: usize,
  pub end_column: usize,
  pub start_offset: usize,
  pub end_offset: usize,
}

#[derive(Debug)]
pub struct NodeInfo {
  pub node_type: NodeType,
  pub is_list: bool,
  pub value: Option<ScalarValue>,
  pub list_values: Vec<ScalarValue>,
  pub location: Location,
  pub key_name: Option<String>,
  pub key_location: Location,
}

#[derive(Debug, Clone)]
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

    check_duplicate_keys(&tree)?;

    Ok(Document { root: tree, path: None })
  }

  pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, YerbaError> {
    let path = path.as_ref();
    let source = fs::read_to_string(path)?;

    let mut document = Self::parse(&source)?;

    document.path = Some(path.to_path_buf());

    Ok(document)
  }

  pub fn save(&self) -> Result<(), YerbaError> {
    let path = self
      .path
      .as_ref()
      .ok_or_else(|| YerbaError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "no file path associated with this document")))?;

    fs::write(path, self.source_text())?;

    Ok(())
  }

  pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), YerbaError> {
    fs::write(path, self.source_text())?;

    Ok(())
  }

  pub fn source(&self, dot_path: &str) -> Result<String, YerbaError> {
    let node = self.navigate(dot_path)?;
    Ok(node.text().to_string())
  }

  pub fn navigate(&self, dot_path: &str) -> Result<SyntaxNode, YerbaError> {
    Self::validate_path(dot_path)?;

    if dot_path.is_empty() {
      let root = Root::cast(self.root.clone()).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

      let document = root.documents().next().ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

      return Ok(document.syntax().clone());
    }

    let nodes = self.navigate_all_compact(dot_path);

    match nodes.len() {
      0 => Err(YerbaError::SelectorNotFound(dot_path.to_string())),
      1 => Ok(nodes.into_iter().next().unwrap()),
      _ => Err(YerbaError::AmbiguousSelector(dot_path.to_string(), nodes.len())),
    }
  }

  pub fn navigate_all_compact(&self, dot_path: &str) -> Vec<SyntaxNode> {
    self.navigate_all(dot_path).into_iter().flatten().collect()
  }

  pub(crate) fn refuse_flow_target(&self, dot_path: &str) -> Result<(), YerbaError> {
    if self.navigate_all_compact(dot_path).iter().any(in_flow_collection) {
      return Err(YerbaError::FlowCollectionNotWritable(dot_path.to_string()));
    }

    Ok(())
  }

  pub fn navigate_all(&self, dot_path: &str) -> Vec<Option<SyntaxNode>> {
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

    let mut current_nodes: Vec<Option<SyntaxNode>> = vec![Some(document.syntax().clone())];

    if parsed.is_empty() {
      if let Some(sequence) = find_block_sequence(document.syntax()) {
        current_nodes = sequence.entries().map(|entry| Some(entry.syntax().clone())).collect();
      }

      return current_nodes;
    }

    let segments = parsed.segments();

    for (i, segment) in segments.iter().enumerate() {
      let is_wildcard = matches!(segment, crate::selector::SelectorSegment::AllItems | crate::selector::SelectorSegment::AllKeys);
      let has_remaining = i + 1 < segments.len();
      let mut next_nodes: Vec<Option<SyntaxNode>> = Vec::new();

      for maybe_node in &current_nodes {
        match maybe_node {
          None => next_nodes.push(None),
          Some(node) => {
            let resolved = resolve_segment(node, segment);

            if is_wildcard && has_remaining {
              let remaining = &segments[i + 1..];

              for item in &resolved {
                let results = navigate_remaining(item, remaining);

                if results.is_empty() {
                  next_nodes.push(None);
                } else {
                  for result in results {
                    next_nodes.push(Some(result));
                  }
                }
              }

              return next_nodes;
            } else if is_wildcard {
              for item in resolved {
                next_nodes.push(Some(item));
              }
            } else if resolved.is_empty() {
              return Vec::new();
            } else {
              for item in resolved {
                next_nodes.push(Some(item));
              }
            }
          }
        }
      }

      current_nodes = next_nodes;

      if current_nodes.iter().all(|n| n.is_none()) {
        break;
      }
    }

    current_nodes
  }

  pub fn validate_path(dot_path: &str) -> Result<(), YerbaError> {
    if dot_path.ends_with('.') {
      return Err(YerbaError::ParseError(format!("invalid path: trailing dot in '{}'", dot_path)));
    }

    if dot_path.contains("..") {
      return Err(YerbaError::ParseError(format!("invalid path: double dot in '{}'", dot_path)));
    }

    if dot_path.starts_with('.') {
      return Err(YerbaError::ParseError(format!("invalid path: leading dot in '{}'", dot_path)));
    }

    if dot_path.contains('[') && !dot_path.contains(']') {
      return Err(YerbaError::ParseError(format!("invalid path: unclosed bracket in '{}'", dot_path)));
    }

    Ok(())
  }

  fn replace_token(&mut self, token: &SyntaxToken, new_text: &str) -> Result<(), YerbaError> {
    let range = token.text_range();

    self.apply_edit(range, new_text)
  }

  fn insert_after_node(&mut self, node: &SyntaxNode, text: &str) -> Result<(), YerbaError> {
    let end: usize = node.text_range().end().into();
    let source = self.source_text();

    let rest = &source[end..];
    let line_end = rest.find('\n').unwrap_or(rest.len());
    let trailing = &rest[..line_end];

    let position = if let Some(comment_start) = trailing.find('#') {
      let _ = comment_start;
      rowan::TextSize::from((end + line_end) as u32)
    } else {
      node.text_range().end()
    };

    let range = TextRange::new(position, position);

    self.apply_edit(range, text)
  }

  fn remove_map_entry(&mut self, entry: &BlockMapEntry) -> Result<(), YerbaError> {
    let entry_node = entry.syntax();
    let entry_range = entry_node.text_range();

    let has_block_value = entry
      .value()
      .map(|value| {
        value
          .syntax()
          .descendants()
          .any(|descendant| descendant.kind() == SyntaxKind::BLOCK_SEQ || descendant.kind() == SyntaxKind::BLOCK_MAP)
      })
      .unwrap_or(false);

    if !has_block_value {
      return self.remove_node(entry_node);
    }

    let source = self.source_text();

    let (start, end) = match preceding_whitespace_token(entry_node) {
      Some(whitespace_token) => {
        let whitespace_text = whitespace_token.text();
        let whitespace_start = whitespace_token.text_range().start();

        let start = whitespace_text
          .rfind('\n')
          .map(|offset| whitespace_start + TextSize::from(offset as u32))
          .unwrap_or(whitespace_start);

        (start, entry_range.end())
      }

      None => {
        let entry_end: usize = entry_range.end().into();
        let line_start = line_start_at(&source, entry_range.start().into());
        let after_entry = source[entry_end..].find('\n').map(|offset| entry_end + offset + 1).unwrap_or(entry_end);

        (TextSize::from(line_start as u32), TextSize::from(after_entry as u32))
      }
    };

    self.apply_edit(TextRange::new(start, end), "")
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

    let indent = entries.get(1).map(|entry| preceding_whitespace_indent(entry.syntax())).unwrap_or_default();

    let text = rebuild_from_groups(&reordered, &indent, true);

    self.apply_edit(range, &text)
  }

  fn apply_edit(&mut self, range: TextRange, replacement: &str) -> Result<(), YerbaError> {
    self.apply_edits(vec![(range, replacement.to_string())])
  }

  fn apply_edits(&mut self, mut edits: Vec<(TextRange, String)>) -> Result<(), YerbaError> {
    if edits.is_empty() {
      return Ok(());
    }

    edits.sort_by_key(|(range, _)| std::cmp::Reverse(range.start()));

    let mut new_source = self.source_text();

    for (range, replacement) in edits {
      let start: usize = range.start().into();
      let end: usize = range.end().into();

      new_source.replace_range(start..end, &replacement);
    }

    self.reparse(&new_source)
  }

  fn source_text(&self) -> String {
    self.root.text().to_string()
  }

  fn reparse(&mut self, new_source: &str) -> Result<(), YerbaError> {
    let document = Self::parse(new_source)?;
    let path = self.path.take();

    *self = document;
    self.path = path;

    Ok(())
  }
}

impl std::fmt::Display for Document {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.root.text())
  }
}

fn check_duplicate_keys(root: &SyntaxNode) -> Result<(), YerbaError> {
  for node in root.descendants() {
    if let Some(map) = BlockMap::cast(node) {
      let mut seen: std::collections::HashMap<String, rowan::TextSize> = std::collections::HashMap::new();

      for entry in map.entries() {
        if let Some(key) = entry.key() {
          if let Some(key_text) = extract_scalar_text(key.syntax()) {
            let offset = key.syntax().text_range().start();

            if let Some(&first_offset) = seen.get(&key_text) {
              let source = root.text().to_string();
              let first_line = line_at(&source, first_offset.into());
              let duplicate_line = line_at(&source, offset.into());
              let line_content = source.lines().nth(duplicate_line - 1).unwrap_or("").to_string();

              return Err(YerbaError::DuplicateKey {
                key: key_text,
                first_line,
                duplicate_line,
                line_content,
              });
            }

            seen.insert(key_text, offset);
          }
        }
      }
    }
  }

  Ok(())
}

pub(crate) fn compute_location(source: &str, start_offset: usize, end_offset: usize) -> Location {
  let start = start_offset.min(source.len());
  let end = end_offset.min(source.len());

  Location {
    start_offset: start,
    end_offset: end,
    start_line: line_at(source, start),
    start_column: column_at(source, start),
    end_line: line_at(source, end),
    end_column: column_at(source, end),
  }
}

pub fn collect_selectors(value: &yaml_serde::Value, prefix: &str, selectors: &mut Vec<String>) {
  match value {
    yaml_serde::Value::Mapping(map) => {
      for (key, child) in map {
        if let yaml_serde::Value::String(key_string) = key {
          let selector = if prefix.is_empty() {
            key_string.clone()
          } else {
            format!("{}.{}", prefix, key_string)
          };

          selectors.push(selector.clone());
          collect_selectors(child, &selector, selectors);
        }
      }
    }

    yaml_serde::Value::Sequence(sequence) => {
      let bracket_prefix = format!("{}[]", prefix);
      selectors.push(bracket_prefix.clone());

      for item in sequence {
        collect_selectors(item, &bracket_prefix, selectors);
      }
    }

    _ => {}
  }
}

pub(crate) fn node_to_yaml_value(node: &SyntaxNode) -> yaml_serde::Value {
  match first_collection(node) {
    Some(FirstCollection::Sequence(sequence)) => {
      let values: Vec<yaml_serde::Value> = sequence.entries().map(|entry| node_to_yaml_value(entry.syntax())).collect();

      return yaml_serde::Value::Sequence(values);
    }

    Some(FirstCollection::Map(map)) => {
      let mut mapping = yaml_serde::Mapping::new();

      for entry in map.entries() {
        let key = entry.key().and_then(|key_node| extract_scalar_text(key_node.syntax())).unwrap_or_default();

        let value = entry
          .value()
          .map(|value_node| node_to_yaml_value(value_node.syntax()))
          .unwrap_or(yaml_serde::Value::Null);

        mapping.insert(yaml_serde::Value::String(key), value);
      }

      return yaml_serde::Value::Mapping(mapping);
    }

    None => {}
  }

  if let Some(block_scalar) = node.descendants().find(|child| child.kind() == SyntaxKind::BLOCK_SCALAR) {
    let text = block_scalar
      .descendants_with_tokens()
      .filter_map(|element| element.into_token())
      .find(|token| token.kind() == SyntaxKind::BLOCK_SCALAR_TEXT)
      .map(|token| token.text().to_string())
      .unwrap_or_default();

    let text = dedent_block_scalar(&text);

    return yaml_serde::Value::String(text);
  }

  if node
    .descendants()
    .any(|descendant| descendant.kind() == SyntaxKind::FLOW_SEQ || descendant.kind() == SyntaxKind::FLOW_MAP)
  {
    let text = node.text().to_string().trim().to_string();

    if let Ok(value) = yaml_serde::from_str(&text) {
      return value;
    }
  }

  if let Some(scalar) = extract_scalar(node) {
    use crate::syntax::{detect_yaml_type, is_yaml_truthy, YerbaValueType};

    return match detect_yaml_type(&scalar) {
      YerbaValueType::Null => yaml_serde::Value::Null,
      YerbaValueType::Boolean => yaml_serde::Value::Bool(is_yaml_truthy(&scalar.text)),

      YerbaValueType::Integer => scalar
        .text
        .parse::<i64>()
        .map(|n| yaml_serde::Value::Number(yaml_serde::Number::from(n)))
        .unwrap_or(yaml_serde::Value::String(scalar.text)),

      YerbaValueType::Float => scalar
        .text
        .parse::<f64>()
        .map(|n| yaml_serde::Value::Number(yaml_serde::Number::from(n)))
        .unwrap_or(yaml_serde::Value::String(scalar.text)),

      YerbaValueType::String => yaml_serde::Value::String(scalar.text),
    };
  }

  let text = node.text().to_string();

  yaml_serde::from_str(&text).unwrap_or(yaml_serde::Value::String(text))
}

pub(crate) fn parse_condition(condition: &str) -> Option<(String, &str, String)> {
  const OPERATORS: [(&str, &str); 4] = [(" not_contains ", "not_contains"), (" contains ", "contains"), ("!=", "!="), ("==", "==")];

  let (left, operator, right) = OPERATORS.iter().find_map(|(pattern, operator)| {
    condition
      .find(pattern)
      .map(|index| (condition[..index].trim(), *operator, condition[index + pattern.len()..].trim()))
  })?;

  let right = right
    .trim_start_matches('"')
    .trim_end_matches('"')
    .trim_start_matches('\'')
    .trim_end_matches('\'');

  Some((left.to_string(), operator, right.to_string()))
}

fn navigate_remaining(node: &SyntaxNode, segments: &[crate::selector::SelectorSegment]) -> Vec<SyntaxNode> {
  let mut current_nodes = vec![node.clone()];

  for segment in segments {
    let mut next_nodes = Vec::new();

    for current in &current_nodes {
      next_nodes.extend(resolve_segment(current, segment));
    }

    if next_nodes.is_empty() {
      return Vec::new();
    }

    current_nodes = next_nodes;
  }

  current_nodes
}

fn resolve_segment(node: &SyntaxNode, segment: &crate::selector::SelectorSegment) -> Vec<SyntaxNode> {
  use crate::selector::SelectorSegment;

  match segment {
    SelectorSegment::AllItems => {
      if let Some(sequence) = find_block_sequence(node) {
        sequence.entries().map(|entry| entry.syntax().clone()).collect()
      } else if let Some(sequence) = find_flow_sequence(node) {
        flow_sequence_entries(&sequence)
      } else {
        Vec::new()
      }
    }

    SelectorSegment::Index(index) => {
      if let Some(sequence) = find_block_sequence(node) {
        sequence.entries().nth(*index).map(|entry| vec![entry.syntax().clone()]).unwrap_or_default()
      } else if let Some(sequence) = find_flow_sequence(node) {
        flow_sequence_entries(&sequence)
          .into_iter()
          .nth(*index)
          .map(|entry| vec![entry])
          .unwrap_or_default()
      } else {
        Vec::new()
      }
    }

    SelectorSegment::AllKeys => {
      if let Some(map) = find_block_map(node) {
        map.entries().filter_map(|entry| entry.value().map(|value| value.syntax().clone())).collect()
      } else if let Some(map) = find_flow_map(node) {
        flow_map_entries(&map)
          .into_iter()
          .filter_map(|entry| entry.value().map(|value| value.syntax().clone()))
          .collect()
      } else {
        Vec::new()
      }
    }

    SelectorSegment::Key(key) => {
      if let Some(map) = find_block_map(node) {
        if let Some(entry) = find_entry_by_key(&map, key) {
          if let Some(value) = entry.value() {
            return vec![value.syntax().clone()];
          }
        }

        return Vec::new();
      }

      if let Some(map) = find_flow_map(node) {
        if let Some(entry) = find_flow_entry_by_key(&map, key) {
          if let Some(value) = entry.value() {
            return vec![value.syntax().clone()];
          }
        }
      }

      Vec::new()
    }
  }
}

pub(crate) fn navigate_from_node(node: &SyntaxNode, path: &str) -> Vec<SyntaxNode> {
  let parsed = crate::selector::Selector::parse(path);

  navigate_remaining(node, parsed.segments())
}

#[derive(Debug, Clone)]
pub(crate) struct EntryGroup {
  pub(crate) separator: String,
  pub(crate) preceding: String,
  pub(crate) body: String,
}

impl EntryGroup {
  pub(crate) fn full_text(&self) -> String {
    if self.preceding.is_empty() {
      self.body.clone()
    } else {
      format!("{}\n{}", self.preceding, self.body)
    }
  }
}

pub(crate) fn collect_blank_line_edits(node: &SyntaxNode, blank_lines: usize, edits: &mut Vec<(TextRange, String)>) {
  use crate::syntax::preceding_whitespace_token;

  if let Some(whitespace_token) = preceding_whitespace_token(node) {
    let whitespace_text = whitespace_token.text();
    let newline_count = whitespace_text.chars().filter(|character| *character == '\n').count();

    let indent = whitespace_text.rfind('\n').map(|position| &whitespace_text[position + 1..]).unwrap_or("");

    let desired_newlines = blank_lines + 1;

    if newline_count != desired_newlines {
      let new_whitespace = format!("{}{}", "\n".repeat(desired_newlines), indent);

      edits.push((whitespace_token.text_range(), new_whitespace));
    }
  }
}

fn collect_entry_groups(parent: &SyntaxNode) -> Vec<EntryGroup> {
  let mut groups: Vec<EntryGroup> = Vec::new();
  let mut buffer = String::new();

  for child in parent.children_with_tokens() {
    let is_entry = child.as_node().is_some() && matches!(child.as_node().unwrap().kind(), SyntaxKind::BLOCK_MAP_ENTRY | SyntaxKind::BLOCK_SEQ_ENTRY);

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

pub(crate) fn collect_preceding_sibling_comments(parent: &SyntaxNode) -> (String, Option<rowan::TextSize>) {
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
      Some(parent) if parent.kind() == SyntaxKind::BLOCK || parent.kind() == SyntaxKind::DOCUMENT || parent.kind() == SyntaxKind::BLOCK_MAP_VALUE => {
        node = parent
      }
      _ => break,
    }
  }

  comments.reverse();
  (comments.join("\n"), earliest_start)
}

pub(crate) fn collect_groups_with_range(parent: &SyntaxNode) -> (Vec<EntryGroup>, TextRange) {
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

pub(crate) fn rebuild_from_groups(groups: &[EntryGroup], indent: &str, preserve_separators: bool) -> String {
  let default_separator = if preserve_separators {
    groups
      .iter()
      .find(|group| !group.separator.is_empty())
      .map(|group| group.separator.clone())
      .unwrap_or_else(|| "\n".to_string())
  } else {
    "\n".to_string()
  };

  groups
    .iter()
    .enumerate()
    .map(|(index, group)| {
      if index == 0 {
        group.full_text()
      } else {
        let separator = if preserve_separators && !group.separator.is_empty() {
          &group.separator
        } else {
          &default_separator
        };

        if group.preceding.is_empty() {
          format!("{}{}{}", separator, indent, group.body)
        } else {
          format!("{}{}\n{}{}", separator, group.preceding, indent, group.body)
        }
      }
    })
    .collect()
}
