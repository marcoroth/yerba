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
    SyntaxKind::DOUBLE_QUOTED_SCALAR => format!("\"{}\"", escape_double_quoted(value)),

    SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let escaped = value.replace('\'', "''");
      format!("'{}'", escaped)
    }

    _ => value.to_string(),
  }
}

pub fn is_control_character(character: char) -> bool {
  matches!(character as u32, 0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f | 0x7f | 0x80..=0x84 | 0x86..=0x9f)
}

pub fn is_single_quotable(value: &str) -> bool {
  !value.contains('\n') && !value.contains('\r') && !value.chars().any(is_control_character)
}

pub fn escape_double_quoted(value: &str) -> String {
  let mut result = String::with_capacity(value.len());

  for character in value.chars() {
    match character {
      '\\' => result.push_str("\\\\"),
      '"' => result.push_str("\\\""),
      '\n' => result.push_str("\\n"),
      '\r' => result.push_str("\\r"),
      '\t' => result.push_str("\\t"),
      _ if is_control_character(character) => result.push_str(&format!("\\x{:02x}", character as u32)),
      _ => result.push(character),
    }
  }

  result
}

const LEADING_INDICATORS: [char; 16] = ['#', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`', ',', '[', ']', '{', '}'];
const FLOW_INDICATORS: [char; 5] = [',', '[', ']', '{', '}'];

pub fn is_plain_safe(value: &str) -> bool {
  if value.is_empty() || value != value.trim() {
    return false;
  }

  if value.contains('\n') || value.contains('\t') {
    return false;
  }

  if value.chars().any(is_control_character) {
    return false;
  }

  if value.contains(": ") || value.ends_with(':') {
    return false;
  }

  if value.contains(" #") {
    return false;
  }

  if value.starts_with("---") || value.starts_with("...") {
    return false;
  }

  let mut characters = value.chars();
  let first = characters.next().expect("value is non-empty");

  if LEADING_INDICATORS.contains(&first) {
    return false;
  }

  if matches!(first, '-' | '?' | ':') {
    return matches!(characters.next(), Some(next) if next != ' ');
  }

  true
}

pub fn is_plain_safe_in_flow(value: &str) -> bool {
  is_plain_safe(value) && !value.contains(FLOW_INDICATORS)
}

pub fn is_quoted_scalar(value: &str) -> bool {
  let bytes = value.as_bytes();

  if bytes.len() < 2 {
    return false;
  }

  match (bytes[0], bytes[bytes.len() - 1]) {
    (b'"', b'"') => {
      let interior = &value[1..value.len() - 1];
      let mut escaped = false;

      for character in interior.chars() {
        if escaped {
          escaped = false;
          continue;
        }

        match character {
          '\\' => escaped = true,
          '"' => return false,
          _ => {}
        }
      }

      !escaped
    }

    (b'\'', b'\'') => {
      let interior = &value[1..value.len() - 1];
      let mut characters = interior.chars().peekable();

      while let Some(character) = characters.next() {
        if character == '\'' {
          if characters.peek() == Some(&'\'') {
            characters.next();
          } else {
            return false;
          }
        }
      }

      true
    }

    _ => false,
  }
}

pub fn is_flow_collection(value: &str) -> bool {
  (value.starts_with('[') && value.ends_with(']')) || (value.starts_with('{') && value.ends_with('}'))
}

pub fn is_raw_yaml_text(value: &str) -> bool {
  value.contains('\n') || value.starts_with("- ") || is_quoted_scalar(value) || is_flow_collection(value)
}

pub fn is_valid_inline_value(value: &str) -> bool {
  if value.is_empty() {
    return true;
  }

  is_raw_yaml_text(value) || is_plain_safe(value)
}

pub fn is_inline_scalar_safe(value: &str) -> bool {
  if value.is_empty() {
    return true;
  }

  if value.contains('\n') || value.starts_with("- ") {
    return false;
  }

  is_quoted_scalar(value) || is_flow_collection(value) || is_plain_safe(value)
}

pub fn needs_quoting(value: &str) -> bool {
  is_yaml_non_string(value) || !is_plain_safe(value)
}

pub fn needs_quoting_in_flow(value: &str) -> bool {
  is_yaml_non_string(value) || !is_plain_safe_in_flow(value)
}

pub fn quote_if_needed(value: &str) -> String {
  if is_raw_yaml_text(value) {
    return value.to_string();
  }

if needs_quoting(value) {
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

fn push_hex_escape(result: &mut String, characters: &mut std::str::Chars<'_>, digits: usize) {
  let escape: String = characters.clone().take(digits).collect();

  let decoded = (escape.len() == digits && escape.chars().all(|character| character.is_ascii_hexdigit()))
    .then(|| u32::from_str_radix(&escape, 16).ok().and_then(char::from_u32))
    .flatten();

  match decoded {
    Some(character) => {
      for _ in 0..digits {
        characters.next();
      }

      result.push(character);
    }

    None => {
      result.push('\\');
      result.push(match digits {
        2 => 'x',
        4 => 'u',
        _ => 'U',
      });
    }
  }
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
        Some('N') => result.push('\u{85}'),
        Some('L') => result.push('\u{2028}'),
        Some('P') => result.push('\u{2029}'),
        Some('x') => push_hex_escape(&mut result, &mut chars, 2),
        Some('u') => push_hex_escape(&mut result, &mut chars, 4),
        Some('U') => push_hex_escape(&mut result, &mut chars, 8),
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
