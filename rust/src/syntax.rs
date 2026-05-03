use rowan::ast::AstNode;
use rowan::{TextRange, TextSize};

use yaml_parser::ast::{BlockMap, BlockMapEntry};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarValue {
  pub text: String,
  pub kind: SyntaxKind,
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

pub fn extract_scalar(node: &SyntaxNode) -> Option<ScalarValue> {
  let token = find_scalar_token(node)?;

  let text = match token.kind() {
    SyntaxKind::PLAIN_SCALAR => token.text().to_string(),

    SyntaxKind::DOUBLE_QUOTED_SCALAR => {
      let raw = token.text();
      unescape_double_quoted(&raw[1..raw.len() - 1])
    }

    SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let raw = token.text();
      unescape_single_quoted(&raw[1..raw.len() - 1])
    }

    _ => return None,
  };

  Some(ScalarValue {
    text,
    kind: token.kind(),
  })
}

pub fn is_map_key(token: &SyntaxToken) -> bool {
  token
    .parent_ancestors()
    .any(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_KEY)
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
  node
    .descendants_with_tokens()
    .filter_map(|element| element.into_token())
    .find(|token| {
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

pub fn extract_scalar_text(node: &SyntaxNode) -> Option<String> {
  let token = find_scalar_token(node)?;

  match token.kind() {
    SyntaxKind::PLAIN_SCALAR => Some(token.text().to_string()),

    SyntaxKind::DOUBLE_QUOTED_SCALAR => {
      let text = token.text();
      let inner = &text[1..text.len() - 1];

      Some(unescape_double_quoted(inner))
    }

    SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let text = token.text();
      let inner = &text[1..text.len() - 1];

      Some(unescape_single_quoted(inner))
    }

    _ => None,
  }
}

pub fn unescape_double_quoted(text: &str) -> String {
  text.replace("\\\"", "\"").replace("\\\\", "\\")
}

pub fn unescape_single_quoted(text: &str) -> String {
  text.replace("''", "'")
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
    let before = &source[..start_offset];

    if let Some(newline_position) = before.rfind('\n') {
      return before[newline_position + 1..].to_string();
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
  matches!(
    value,
    "true" | "True" | "TRUE" | "yes" | "Yes" | "YES" | "on" | "On" | "ON" | "y" | "Y"
  )
}

pub fn detect_yaml_type_from_plain(value: &str) -> YerbaValueType {
  // Null (YAML 1.1 + 1.2)
  if matches!(value, "null" | "Null" | "NULL" | "~" | "") {
    return YerbaValueType::Null;
  }

  // Boolean (YAML 1.2 + 1.1)
  if matches!(
    value,
    "true"
      | "True"
      | "TRUE"
      | "false"
      | "False"
      | "FALSE"
      | "yes"
      | "Yes"
      | "YES"
      | "no"
      | "No"
      | "NO"
      | "on"
      | "On"
      | "ON"
      | "off"
      | "Off"
      | "OFF"
      | "y"
      | "Y"
      | "n"
      | "N"
  ) {
    return YerbaValueType::Boolean;
  }

  // Integer
  if value.parse::<i64>().is_ok() {
    return YerbaValueType::Integer;
  }

  // Octal (0o...) and hex (0x...)
  if value.starts_with("0x") || value.starts_with("0X") || value.starts_with("0o") || value.starts_with("0O") {
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
