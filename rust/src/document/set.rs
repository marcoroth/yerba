use super::*;

impl Document {
  pub fn set(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    if let Some(block_scalar) = current_node.descendants().find(|node| node.kind() == SyntaxKind::BLOCK_SCALAR) {
      let new_text = if value.is_empty() {
        "\"\"".to_string()
      } else if value.contains('\n') {
        let source = self.to_string();
        let offset: usize = block_scalar.text_range().start().into();
        let line_start = source[..offset].rfind('\n').map(|position| position + 1).unwrap_or(0);
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
      } else {
        format!("\"{}\"", value.replace('"', "\\\""))
      };

      let range = block_scalar.text_range();

      return self.apply_edit(range, &new_text);
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

    let source = self.to_string();
    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for node in nodes {
      if let Some(block_scalar) = node.descendants().find(|child| child.kind() == SyntaxKind::BLOCK_SCALAR) {
        let new_text = if value.is_empty() {
          "\"\"".to_string()
        } else if value.contains('\n') {
          let offset: usize = block_scalar.text_range().start().into();
          let line_start = source[..offset].rfind('\n').map(|position| position + 1).unwrap_or(0);
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
        } else {
          format!("\"{}\"", value.replace('"', "\\\""))
        };

        edits.push((block_scalar.text_range(), new_text));
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

    if let Some(block_scalar) = current_node.descendants().find(|node| node.kind() == SyntaxKind::BLOCK_SCALAR) {
      let range = block_scalar.text_range();
      return self.apply_edit(range, value);
    }

    let scalar_token = find_scalar_token(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    self.replace_token(&scalar_token, value)
  }
}
