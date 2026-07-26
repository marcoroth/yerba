use rowan::ast::AstNode;
use rowan::{TextRange, TextSize};

use yaml_parser::ast::{BlockMap, BlockMapEntry, BlockSeq, FlowMap, FlowMapEntry, FlowSeq};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarValue {
  pub text: String,
  pub kind: SyntaxKind,
  pub file_path: Option<String>,
  pub selector: Option<String>,
  pub line: Option<usize>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YerbaValueType {
  Null = 0,
  Boolean = 1,
  Integer = 2,
  Float = 3,
  String = 4,
}

pub fn detect_yaml_type(scalar: &ScalarValue) -> YerbaValueType {
  if scalar.kind != SyntaxKind::PLAIN_SCALAR {
    return YerbaValueType::String;
  }

  detect_yaml_type_from_plain(&scalar.text)
}

pub fn raw_scalar_value(token: &SyntaxToken) -> Option<String> {
  match token.kind() {
    SyntaxKind::PLAIN_SCALAR => Some(token.text().to_string()),

    SyntaxKind::DOUBLE_QUOTED_SCALAR => {
      let text = token.text();
      Some(unescape_double_quoted(&text[1..text.len() - 1]))
    }

    SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let text = token.text();
      Some(unescape_single_quoted(&text[1..text.len() - 1]))
    }

    _ => None,
  }
}

pub fn extract_scalar(node: &SyntaxNode) -> Option<ScalarValue> {
  if let Some(token) = find_scalar_token(node) {
    return raw_scalar_value(&token).map(|text| ScalarValue {
      text,
      kind: token.kind(),
      file_path: None,
      selector: None,
      line: None,
    });
  }

  let block_token = node
    .descendants_with_tokens()
    .filter_map(|element| element.into_token())
    .find(|token| token.kind() == SyntaxKind::BLOCK_SCALAR_TEXT)?;

  Some(ScalarValue {
    text: dedent_block_scalar(block_token.text()),
    kind: SyntaxKind::BLOCK_SCALAR_TEXT,
    file_path: None,
    selector: None,
    line: None,
  })
}

pub fn find_block_map(node: &SyntaxNode) -> Option<BlockMap> {
  node.descendants().find_map(BlockMap::cast)
}

pub fn find_block_sequence(node: &SyntaxNode) -> Option<BlockSeq> {
  node.descendants().find_map(BlockSeq::cast)
}

fn first_collection_node(node: &SyntaxNode) -> Option<SyntaxNode> {
  node.descendants().find(|descendant| {
    matches!(
      descendant.kind(),
      SyntaxKind::BLOCK_MAP | SyntaxKind::BLOCK_SEQ | SyntaxKind::FLOW_MAP | SyntaxKind::FLOW_SEQ
    )
  })
}

pub fn find_flow_sequence(node: &SyntaxNode) -> Option<FlowSeq> {
  first_collection_node(node).and_then(FlowSeq::cast)
}

pub fn find_flow_map(node: &SyntaxNode) -> Option<FlowMap> {
  first_collection_node(node).and_then(FlowMap::cast)
}

pub fn flow_sequence_entries(sequence: &FlowSeq) -> Vec<SyntaxNode> {
  sequence
    .entries()
    .map(|entries| entries.entries().map(|entry| entry.syntax().clone()).collect())
    .unwrap_or_default()
}

pub fn flow_map_entries(map: &FlowMap) -> Vec<FlowMapEntry> {
  map.entries().map(|entries| entries.entries().collect()).unwrap_or_default()
}

pub fn find_flow_entry_by_key(map: &FlowMap, key: &str) -> Option<FlowMapEntry> {
  flow_map_entries(map)
    .into_iter()
    .find(|entry| entry.key().and_then(|found| extract_scalar_text(found.syntax())).as_deref() == Some(key))
}

pub fn in_flow_collection(node: &SyntaxNode) -> bool {
  node
    .ancestors()
    .any(|ancestor| matches!(ancestor.kind(), SyntaxKind::FLOW_MAP | SyntaxKind::FLOW_SEQ))
}

pub enum FirstCollection {
  Map(BlockMap),
  Sequence(BlockSeq),
}

pub fn first_collection(node: &SyntaxNode) -> Option<FirstCollection> {
  match (find_block_map(node), find_block_sequence(node)) {
    (Some(map), Some(sequence)) => {
      if sequence.syntax().text_range().start() <= map.syntax().text_range().start() {
        Some(FirstCollection::Sequence(sequence))
      } else {
        Some(FirstCollection::Map(map))
      }
    }

    (Some(map), None) => Some(FirstCollection::Map(map)),
    (None, Some(sequence)) => Some(FirstCollection::Sequence(sequence)),
    (None, None) => None,
  }
}

pub fn is_map_key(token: &SyntaxToken) -> bool {
  token.parent_ancestors().any(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_KEY)
}

pub fn find_entry_by_key(map: &BlockMap, key: &str) -> Option<BlockMapEntry> {
  map.entries().find(|entry| {
    entry
      .key()
      .and_then(|key_node| extract_scalar_text(key_node.syntax()))
      .map(|key_text| key_text == key)
      .unwrap_or(false)
  })
}

pub fn find_scalar_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node.descendants_with_tokens().filter_map(|element| element.into_token()).find(|token| {
    matches!(
      token.kind(),
      SyntaxKind::PLAIN_SCALAR | SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR
    )
  })
}

pub fn format_scalar_value(value: &str, kind: SyntaxKind) -> String {
  match kind {
    SyntaxKind::DOUBLE_QUOTED_SCALAR => {
      let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
      format!("\"{}\"", escaped)
    }

    SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let escaped = value.replace('\'', "''");
      format!("'{}'", escaped)
    }

    _ => value.to_string(),
  }
}

pub fn quote_if_needed(value: &str) -> String {
  if is_yaml_non_string(value) {
    format_scalar_value(value, SyntaxKind::DOUBLE_QUOTED_SCALAR)
  } else {
    value.to_string()
  }
}

pub fn extract_scalar_text(node: &SyntaxNode) -> Option<String> {
  extract_scalar(node).map(|scalar| scalar.text)
}

pub fn dedent_block_scalar(text: &str) -> String {
  let lines: Vec<&str> = text.lines().collect();
  let min_indent = lines
    .iter()
    .filter(|line| !line.trim().is_empty())
    .map(|line| line.len() - line.trim_start().len())
    .min()
    .unwrap_or(0);

  let dedented: String = lines
    .iter()
    .map(|line| if line.len() >= min_indent { &line[min_indent..] } else { line.trim() })
    .collect::<Vec<_>>()
    .join("\n");

  dedented.trim().to_string()
}

pub fn unescape_double_quoted(text: &str) -> String {
  let mut result = String::with_capacity(text.len());
  let mut chars = text.chars();

  while let Some(character) = chars.next() {
    if character == '\\' {
      match chars.next() {
        Some('n') => result.push('\n'),
        Some('t') => result.push('\t'),
        Some('r') => result.push('\r'),
        Some('\\') => result.push('\\'),
        Some('"') => result.push('"'),
        Some('/') => result.push('/'),
        Some('0') => result.push('\0'),
        Some('a') => result.push('\u{07}'),
        Some('b') => result.push('\u{08}'),
        Some('e') => result.push('\u{1b}'),
        Some('v') => result.push('\u{0b}'),
        Some(' ') => result.push(' '),
        Some('_') => result.push('\u{a0}'),
        Some('\n') => {} // line continuation: skip newline and leading whitespace
        Some(other) => {
          result.push('\\');
          result.push(other);
        }
        None => result.push('\\'),
      }
    } else {
      result.push(character);
    }
  }

  result
}

pub fn unescape_single_quoted(text: &str) -> String {
  text.replace("''", "'")
}

pub fn line_at(source: &str, offset: usize) -> usize {
  source[..offset].matches('\n').count() + 1
}

pub fn line_start_at(source: &str, offset: usize) -> usize {
  source[..offset].rfind('\n').map(|position| position + 1).unwrap_or(0)
}

pub fn column_at(source: &str, offset: usize) -> usize {
  offset - line_start_at(source, offset)
}

pub fn preceding_whitespace_indent(node: &SyntaxNode) -> String {
  if let Some(token) = preceding_whitespace_token(node) {
    let text = token.text();

    if let Some(newline) = text.rfind('\n') {
      return text[newline + 1..].to_string();
    }
  }

  let start_offset: usize = node.text_range().start().into();
  let root = node.ancestors().last().unwrap_or_else(|| node.clone());
  let source = root.text().to_string();

  if start_offset > 0 {
    let line_start = line_start_at(&source, start_offset);

    if line_start > 0 {
      return source[line_start..start_offset].to_string();
    }
  }

  String::new()
}

pub fn preceding_whitespace_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .prev_sibling_or_token()
    .and_then(|sibling| sibling.into_token())
    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
}

pub fn following_whitespace_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .next_sibling_or_token()
    .and_then(|sibling| sibling.into_token())
    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
}

pub fn removal_range(node: &SyntaxNode) -> TextRange {
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

pub fn is_yaml_non_string(value: &str) -> bool {
  detect_yaml_type_from_plain(value) != YerbaValueType::String
}

pub fn is_yaml_truthy(value: &str) -> bool {
  matches!(value, "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "on" | "On" | "ON")
}

pub fn detect_yaml_type_from_plain(value: &str) -> YerbaValueType {
  // Null (YAML 1.1 + 1.2)
  if matches!(value, "null" | "Null" | "NULL" | "~" | "") {
    return YerbaValueType::Null;
  }

  // Boolean, resolved the way libyaml/Psych resolve it: the YAML 1.1 bool set
  // without the single-letter y/Y/n/N forms (which libyaml treats as strings),
  // and broader than YAML 1.2 core schema (which only accepts true/false).
  if matches!(
    value,
    "true" | "True" | "TRUE" | "false" | "False" | "FALSE" | "yes" | "Yes" | "YES" | "no" | "No" | "NO" | "on" | "On" | "ON" | "off" | "Off" | "OFF"
  ) {
    return YerbaValueType::Boolean;
  }

  // Integer
  if value.parse::<i64>().is_ok() {
    return YerbaValueType::Integer;
  }

  // Hex (0x...) — only valid hex digits after prefix
  if (value.starts_with("0x") || value.starts_with("0X")) && value.len() > 2 && value[2..].chars().all(|c| c.is_ascii_hexdigit()) {
    return YerbaValueType::Integer;
  }

  // Octal (0o...) — only valid octal digits after prefix
  if (value.starts_with("0o") || value.starts_with("0O")) && value.len() > 2 && value[2..].chars().all(|c| matches!(c, '0'..='7')) {
    return YerbaValueType::Integer;
  }

  // Special floats (YAML 1.1 + 1.2)
  if matches!(
    value,
    ".inf" | ".Inf" | ".INF" | "-.inf" | "-.Inf" | "-.INF" | "+.inf" | "+.Inf" | "+.INF" | ".nan" | ".NaN" | ".NAN"
  ) {
    return YerbaValueType::Float;
  }

  // Float
  if value.parse::<f64>().is_ok() {
    return YerbaValueType::Float;
  }

  YerbaValueType::String
}
