use rowan::ast::AstNode;
use rowan::{TextRange, TextSize};

use yaml_parser::ast::{BlockMap, BlockMapEntry};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

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
        SyntaxKind::PLAIN_SCALAR
          | SyntaxKind::DOUBLE_QUOTED_SCALAR
          | SyntaxKind::SINGLE_QUOTED_SCALAR
      )
    })
}

pub fn format_scalar_value(value: &str, kind: SyntaxKind) -> String {
  match kind {
    SyntaxKind::DOUBLE_QUOTED_SCALAR => format!("\"{}\"", value),
    SyntaxKind::SINGLE_QUOTED_SCALAR => format!("'{}'", value),
    _ => value.to_string(),
  }
}

pub fn extract_scalar_text(node: &SyntaxNode) -> Option<String> {
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

pub fn preceding_whitespace_indent(node: &SyntaxNode) -> String {
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
