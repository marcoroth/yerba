use super::*;

impl Document {
  pub fn enforce_blank_lines(&mut self, dot_path: &str, blank_lines: usize) -> Result<(), YerbaError> {
    let nodes = if dot_path.contains('[') {
      self.navigate_all(dot_path)
    } else {
      vec![self.navigate(dot_path)?]
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for current_node in &nodes {
      let sequence = current_node.descendants().find_map(BlockSeq::cast);
      let map = current_node.descendants().find_map(BlockMap::cast);

      let use_sequence = match (&sequence, &map) {
        (Some(sequence), Some(map)) => sequence.syntax().text_range().start() <= map.syntax().text_range().start(),
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (None, None) => continue,
      };

      if use_sequence {
        let entries: Vec<_> = sequence.unwrap().entries().collect();

        if entries.len() > 1 {
          for entry in entries.iter().skip(1) {
            collect_blank_line_edits(entry.syntax(), blank_lines, &mut edits);
          }
        }
      } else {
        let entries: Vec<_> = map.unwrap().entries().collect();

        if entries.len() > 1 {
          for entry in entries.iter().skip(1) {
            collect_blank_line_edits(entry.syntax(), blank_lines, &mut edits);
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

  pub fn has_directives_marker(&self) -> bool {
    self.root.descendants_with_tokens().any(|element| element.kind() == SyntaxKind::DIRECTIVES_END)
  }

  pub fn ensure_directives(&mut self) -> Result<(), YerbaError> {
    if self.has_directives_marker() {
      return Ok(());
    }

    let source = self.root.text().to_string();
    let new_source = format!("---\n{}", source);

    let path = self.path.take();
    *self = Self::parse(&new_source)?;
    self.path = path;

    Ok(())
  }

  pub fn remove_directives(&mut self) -> Result<(), YerbaError> {
    if !self.has_directives_marker() {
      return Ok(());
    }

    let source = self.root.text().to_string();
    let new_source = source
      .strip_prefix("---\n")
      .or_else(|| source.strip_prefix("---"))
      .unwrap_or(&source)
      .to_string();

    let path = self.path.take();
    *self = Self::parse(&new_source)?;
    self.path = path;

    Ok(())
  }

  pub fn enforce_key_style(&mut self, style: &crate::KeyStyle, dot_path: Option<&str>) -> Result<(), YerbaError> {
    let source = self.root.text().to_string();

    let scope_ranges: Vec<TextRange> = match dot_path {
      Some(path) if !path.is_empty() => self.navigate_all(path).iter().map(|node| node.text_range()).collect(),
      _ => vec![self.root.text_range()],
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for element in self.root.descendants_with_tokens() {
      if let Some(token) = element.into_token() {
        if !scope_ranges.iter().any(|range| range.contains_range(token.text_range())) {
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
          crate::KeyStyle::Double => {
            let escaped = raw_value.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
          }

          crate::KeyStyle::Single => {
            let escaped = raw_value.replace('\'', "''");
            format!("'{}'", escaped)
          }

          crate::KeyStyle::Plain => raw_value,
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

  pub fn enforce_quote_style(
    &mut self,
    key_style: Option<&crate::KeyStyle>,
    value_style: Option<&QuoteStyle>,
    selector: Option<&str>,
  ) -> Result<(), YerbaError> {
    if let Some(key_style) = key_style {
      self.enforce_key_style(key_style, selector)?;
    }

    if let Some(value_style) = value_style {
      self.enforce_quotes_at(value_style, selector)?;
    }

    Ok(())
  }

  pub fn enforce_quotes(&mut self, style: &QuoteStyle) -> Result<Vec<String>, YerbaError> {
    self.enforce_quotes_at(style, None)
  }

  pub fn enforce_quotes_at(&mut self, style: &QuoteStyle, dot_path: Option<&str>) -> Result<Vec<String>, YerbaError> {
    let source = self.root.text().to_string();

    let scope_ranges: Vec<TextRange> = match dot_path {
      Some(path) if !path.is_empty() => self.navigate_all(path).iter().map(|node| node.text_range()).collect(),
      _ => vec![self.root.text_range()],
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for element in self.root.descendants_with_tokens() {
      if let Some(token) = element.into_token() {
        if !scope_ranges.iter().any(|range| range.contains_range(token.text_range())) {
          continue;
        }

        if is_map_key(&token) {
          continue;
        }

        let current_kind = token.kind();
        let is_block_scalar_target = style.is_block_scalar();

        let is_inline_scalar = matches!(
          current_kind,
          SyntaxKind::PLAIN_SCALAR | SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR
        );

        let is_block_scalar_text = current_kind == SyntaxKind::BLOCK_SCALAR_TEXT;

        if !is_inline_scalar && !is_block_scalar_text {
          continue;
        }

        if is_block_scalar_text && is_block_scalar_target {
          if let Some(parent) = token.parent() {
            if parent.kind() == SyntaxKind::BLOCK_SCALAR {
              let parent_text = parent.text().to_string();
              let target_header = style.block_header();
              let current_header = parent_text.lines().next().unwrap_or("");

              if current_header != target_header {
                let content = parent_text.strip_prefix(current_header).unwrap_or(&parent_text);
                let new_text = format!("{}{}", target_header, content);

                edits.push((parent.text_range(), new_text));
              }
            }
          }

          continue;
        }

        if is_block_scalar_text && !is_block_scalar_target {
          if dot_path.is_none() || dot_path == Some("") {
            continue;
          }

          if let Some(parent) = token.parent() {
            if parent.kind() == SyntaxKind::BLOCK_SCALAR {
              let raw_text = token.text().to_string();
              let lines: Vec<&str> = raw_text.lines().collect();

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

              let trimmed = dedented.trim();
              let is_multiline = trimmed.contains('\n');

              let new_text = match style {
                QuoteStyle::Double => {
                  let escaped = trimmed.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");

                  format!("\"{}\"", escaped)
                }

                QuoteStyle::Single => {
                  if is_multiline {
                    let offset: usize = token.text_range().start().into();
                    let line = source[..offset].matches('\n').count() + 1;

                    warnings.push(format!(
                      "line {}: skipped block scalar → single (multiline content can't be single-quoted)",
                      line
                    ));

                    continue;
                  }

                  let escaped = trimmed.replace('\'', "''");

                  format!("'{}'", escaped)
                }

                QuoteStyle::Plain => {
                  if is_multiline || trimmed.contains(':') || trimmed.contains('#') || trimmed.contains('"') || trimmed.contains('\'') {
                    let offset: usize = token.text_range().start().into();
                    let line = source[..offset].matches('\n').count() + 1;
                    let reason = if is_multiline { "multiline content" } else { "special characters" };

                    warnings.push(format!("line {}: skipped block scalar → plain ({} can't be plain)", line, reason));

                    continue;
                  }

                  trimmed.to_string()
                }

                _ => continue,
              };

              edits.push((parent.text_range(), new_text));
            }
          }

          continue;
        }

        if is_inline_scalar && is_block_scalar_target {
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

          if raw_value.is_empty() || is_yaml_non_string(&raw_value) {
            continue;
          }

          let offset: usize = token.text_range().start().into();
          let line_start = source[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
          let line_prefix = &source[line_start..offset];
          let indent = line_prefix.len() - line_prefix.trim_start().len() + 2;
          let indent_str = " ".repeat(indent);
          let header = style.block_header();

          let indented = raw_value
            .lines()
            .map(|line| if line.is_empty() { String::new() } else { format!("{}{}", indent_str, line) })
            .collect::<Vec<_>>()
            .join("\n");

          let new_text = format!("{}\n{}", header, indented);

          edits.push((token.text_range(), new_text));

          continue;
        }

        if !is_inline_scalar {
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
            if raw_value.contains('"') || raw_value.contains('\'') || raw_value.contains(':') || raw_value.contains('#') {
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
      return Ok(warnings);
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

    Ok(warnings)
  }
}
