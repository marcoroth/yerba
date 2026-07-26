use super::*;

fn holds_collection(node: &SyntaxNode) -> bool {
  node.descendants().any(|descendant| {
    matches!(
      descendant.kind(),
      SyntaxKind::BLOCK_MAP | SyntaxKind::BLOCK_SEQ | SyntaxKind::FLOW_MAP | SyntaxKind::FLOW_SEQ
    )
  })
}

struct ValueSpan {
  range: TextRange,
  separated: bool,
  comment: Option<String>,
}

fn value_span(node: &SyntaxNode) -> Option<ValueSpan> {
  let (container, separator_kind) = match node.kind() {
    SyntaxKind::BLOCK_MAP_VALUE | SyntaxKind::FLOW_MAP_VALUE => (node.parent()?, SyntaxKind::COLON),
    SyntaxKind::BLOCK_SEQ_ENTRY => (node.clone(), SyntaxKind::MINUS),

    SyntaxKind::FLOW_SEQ_ENTRY => {
      return Some(ValueSpan {
        range: node.text_range(),
        separated: false,
        comment: None,
      })
    }

    _ => return None,
  };

  let mut elements = container
    .children_with_tokens()
    .skip_while(|element| element.as_token().map(|token| token.kind() != separator_kind).unwrap_or(true));

  let separator = elements.next()?.into_token()?;
  let mut comment = None;

  for element in elements {
    match element.into_token() {
      Some(token) if token.kind() == SyntaxKind::COMMENT => comment = Some(token.text().trim_end().to_string()),
      Some(token) if token.kind() == SyntaxKind::WHITESPACE && token.text().contains('\n') => break,
      Some(_) => {}
      None => break,
    }
  }

  Some(ValueSpan {
    range: TextRange::new(separator.text_range().end(), node.text_range().end()),
    separated: true,
    comment,
  })
}

fn replacement_text(span: &ValueSpan, value: &str) -> String {
  let mut text = if span.separated { format!(" {}", value) } else { value.to_string() };

  if let Some(comment) = &span.comment {
    text.push(' ');
    text.push_str(comment);
  }

  text
}

impl Document {
  pub fn set(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    if let Some(block_scalar) = current_node.descendants().find(|node| node.kind() == SyntaxKind::BLOCK_SCALAR) {
      let source = self.source_text();
      let new_text = Self::block_scalar_replacement(&source, &block_scalar, value);

      return self.apply_edit(block_scalar.text_range(), &new_text);
    }

    if holds_collection(&current_node) {
      let span = value_span(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

      return self.apply_edit(span.range, &replacement_text(&span, value));
    }

    let scalar_token = find_scalar_token(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    let new_text = format_scalar_value(value, scalar_token.kind());

    self.replace_token(&scalar_token, &new_text)
  }

  pub fn set_all(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let nodes = self.navigate_all_compact(dot_path);

    if nodes.is_empty() {
      return Err(YerbaError::SelectorNotFound(dot_path.to_string()));
    }

    let source = self.source_text();
    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for node in nodes {
      if let Some(block_scalar) = node.descendants().find(|child| child.kind() == SyntaxKind::BLOCK_SCALAR) {
        edits.push((block_scalar.text_range(), Self::block_scalar_replacement(&source, &block_scalar, value)));
      } else if holds_collection(&node) {
        if let Some(span) = value_span(&node) {
          edits.push((span.range, replacement_text(&span, value)));
        }
      } else if let Some(scalar_token) = find_scalar_token(&node) {
        edits.push((scalar_token.text_range(), format_scalar_value(value, scalar_token.kind())));
      }
    }

    self.apply_edits(edits)
  }

  pub fn set_scalar_style(&mut self, dot_path: &str, style: &QuoteStyle) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;
    let scalar_token = find_scalar_token(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    let current_kind = scalar_token.kind();
    let target_kind = style.to_syntax_kind();

    if current_kind == target_kind {
      return Ok(());
    }

    let raw_value = match raw_scalar_value(&scalar_token) {
      Some(value) => value,
      None => return Ok(()),
    };

    let new_text = format_scalar_value(&raw_value, target_kind);

    self.replace_token(&scalar_token, &new_text)
  }

  pub fn set_plain(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    if let Some(block_scalar) = current_node.descendants().find(|node| node.kind() == SyntaxKind::BLOCK_SCALAR) {
      let range = block_scalar.text_range();
      return self.apply_edit(range, value);
    }

    if holds_collection(&current_node) {
      let span = value_span(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

      return self.apply_edit(span.range, &replacement_text(&span, value));
    }

    let scalar_token = find_scalar_token(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    self.replace_token(&scalar_token, value)
  }

  fn block_scalar_replacement(source: &str, block_scalar: &SyntaxNode, value: &str) -> String {
    if value.is_empty() {
      return "\"\"".to_string();
    }

    if !value.contains('\n') {
      return format!("\"{}\"", value.replace('"', "\\\""));
    }

    let offset: usize = block_scalar.text_range().start().into();
    let line_start = line_start_at(source, offset);
    let key_indent = source[line_start..offset].len() - source[line_start..offset].trim_start().len();
    let indent = " ".repeat(key_indent + 2);

    let indented_lines: Vec<String> = value
      .split('\n')
      .enumerate()
      .map(|(index, line)| {
        if line.is_empty() && index > 0 {
          String::new()
        } else {
          format!("{}{}", indent, line)
        }
      })
      .collect();

    format!("|-\n{}", indented_lines.join("\n"))
  }
}
