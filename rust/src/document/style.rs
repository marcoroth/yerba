use super::*;

pub struct StyleEnforcement {
  pub collection_style: Option<String>,
  pub sequence_indent: Option<String>,
  pub key_style: Option<crate::KeyStyle>,
  pub value_style: Option<QuoteStyle>,
}

impl Document {
  pub fn enforce_styles(&mut self, enforcement: &StyleEnforcement) -> Result<(), YerbaError> {
    let source = self.source_text();
    let mut edits: Vec<(TextRange, String)> = Vec::new();
    let mut converted_ranges: Vec<TextRange> = Vec::new();

    let want_block_collections = enforcement.collection_style.as_deref() == Some("block");
    let want_indented = enforcement.sequence_indent.as_deref() == Some("indented");
    let want_compact = enforcement.sequence_indent.as_deref() == Some("compact");

    for element in self.root.descendants_with_tokens() {
      match element {
        rowan::NodeOrToken::Node(ref node) => {
          if want_block_collections && (node.kind() == SyntaxKind::FLOW_SEQ || node.kind() == SyntaxKind::FLOW_MAP) {
            if node
              .ancestors()
              .skip(1)
              .any(|ancestor| ancestor.kind() == SyntaxKind::FLOW_SEQ || ancestor.kind() == SyntaxKind::FLOW_MAP)
            {
              continue;
            }

            let value = node_to_yaml_value(node);

            match &value {
              yaml_serde::Value::Sequence(sequence) if sequence.is_empty() => continue,
              yaml_serde::Value::Mapping(mapping) if mapping.is_empty() => continue,
              _ => {}
            }

            let entry_node = node.ancestors().find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY);

            if let Some(ref entry) = entry_node {
              let entry_start: usize = entry.text_range().start().into();
              let key_indent = column_at(&source, entry_start);
              let entry_text = entry.text().to_string();

              if let Some(colon_offset) = entry_text.find(':') {
                let colon_position = entry_start + colon_offset;
                let value_indent = key_indent + 2;
                let block_text = crate::yaml_writer::yaml_value_to_block_text(&value, value_indent);
                let replace_range = TextRange::new(rowan::TextSize::from((colon_position + 1) as u32), node.text_range().end());

                converted_ranges.push(node.text_range());
                edits.push((replace_range, format!("\n{}", block_text)));
              }
            }
          }

          if (want_indented || want_compact) && BlockSeq::can_cast(node.kind()) {
            if node.ancestors().skip(1).any(|ancestor| BlockSeq::can_cast(ancestor.kind())) {
              continue;
            }

            let parent_entry = match node.ancestors().find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY) {
              Some(entry) => entry,
              None => continue,
            };

            let sequence = match BlockSeq::cast(node.clone()) {
              Some(sequence) => sequence,
              None => continue,
            };

            let first_entry = match sequence.entries().next() {
              Some(entry) => entry,
              None => continue,
            };

            let entry_start: usize = parent_entry.text_range().start().into();
            let key_indent_length = column_at(&source, entry_start);
            let entry_indent_length = preceding_whitespace_indent(first_entry.syntax()).len();
            let is_indented = entry_indent_length > key_indent_length;
            let needs_change = (want_indented && !is_indented) || (want_compact && is_indented);

            if needs_change {
              let target_style = enforcement.sequence_indent.as_deref().unwrap();

              let indent_diff: i32 = match target_style {
                "indented" => (key_indent_length as i32 + 2) - entry_indent_length as i32,
                "compact" => key_indent_length as i32 - entry_indent_length as i32,
                _ => continue,
              };

              if indent_diff == 0 {
                continue;
              }

              let sequence_range = node.text_range();
              let sequence_start: usize = sequence_range.start().into();
              let sequence_end: usize = sequence_range.end().into();

              let line_start_of_first_entry = line_start_at(&source, sequence_start);
              let full_text = &source[line_start_of_first_entry..sequence_end];

              let reindented: String = full_text
                .lines()
                .map(|line| {
                  if line.trim().is_empty() {
                    String::new()
                  } else {
                    let current_line_indent = line.len() - line.trim_start().len();
                    let new_indent = (current_line_indent as i32 + indent_diff).max(0) as usize;
                    format!("{}{}", " ".repeat(new_indent), line.trim_start())
                  }
                })
                .collect::<Vec<_>>()
                .join("\n");

              let replace_range = TextRange::new(rowan::TextSize::from(line_start_of_first_entry as u32), sequence_range.end());

              edits.push((replace_range, reindented));
            }
          }
        }

        rowan::NodeOrToken::Token(ref token) => {
          if converted_ranges.iter().any(|range| range.contains_range(token.text_range())) {
            continue;
          }

          let current_kind = token.kind();

          if let Some(ref key_style) = enforcement.key_style {
            if is_map_key(token)
              && matches!(
                current_kind,
                SyntaxKind::PLAIN_SCALAR | SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR
              )
            {
              let target_kind = key_style.to_syntax_kind();

              if current_kind != target_kind {
                let raw_value = match raw_scalar_value(token) {
                  Some(value) => value,
                  None => continue,
                };

                let new_text = format_scalar_value(&raw_value, target_kind);

                if new_text != token.text() {
                  edits.push((token.text_range(), new_text));
                }
              }

              continue;
            }
          }

          if let Some(ref value_style) = enforcement.value_style {
            if is_map_key(token) {
              continue;
            }

            let is_inline_scalar = matches!(
              current_kind,
              SyntaxKind::PLAIN_SCALAR | SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR
            );

            if !is_inline_scalar {
              continue;
            }

            if value_style.is_block_scalar() {
              continue;
            }

            let target_kind = value_style.to_syntax_kind();

            if current_kind == target_kind {
              continue;
            }

            let raw_value = match raw_scalar_value(token) {
              Some(value) => value,
              None => continue,
            };

            if current_kind == SyntaxKind::PLAIN_SCALAR && is_yaml_non_string(&raw_value) {
              continue;
            }

            let new_text = match inline_quote_replacement(&raw_value, current_kind, value_style) {
              Some(new_text) => new_text,
              None => continue,
            };

            if new_text != token.text() {
              edits.push((token.text_range(), new_text));
            }
          }
        }
      }
    }

    self.apply_edits(edits)
  }

  pub fn enforce_collection_style(&mut self, style: &str, dot_path: Option<&str>) -> Result<(), YerbaError> {
    let scope_path = dot_path.unwrap_or("");

    if scope_path.contains("[]") {
      let concrete_selectors = self.resolve_selectors(scope_path);

      for selector in concrete_selectors.iter().rev() {
        self.enforce_collection_style(style, Some(selector))?;
      }

      return Ok(());
    }

    match style {
      "block" | "flow" => {}
      _ => {
        return Err(YerbaError::ParseError(format!(
          "unknown collection style '{}'. Valid options: flow, block",
          style
        )))
      }
    }

    if style == "block" {
      let has_flow = self
        .root
        .descendants_with_tokens()
        .any(|element| matches!(element.kind(), SyntaxKind::FLOW_SEQ | SyntaxKind::FLOW_MAP));

      if !has_flow {
        return Ok(());
      }
    }

    let value = match self.get_value(scope_path) {
      Some(value) => value,
      None => return Ok(()),
    };

    let mut selectors = Vec::new();

    if !scope_path.is_empty() {
      selectors.push(scope_path.to_string());
    }

    collect_selectors(&value, scope_path, &mut selectors);

    let mut collection_selectors: Vec<String> = selectors
      .into_iter()
      .filter(|selector| self.get_collection_style(selector).is_some_and(|current_style| current_style != style))
      .collect();

    collection_selectors.sort_by_key(|selector| std::cmp::Reverse(selector.len()));

    for selector in &collection_selectors {
      self.set_collection_style(selector, style)?;
    }

    Ok(())
  }

  pub fn enforce_sequence_indent(&mut self, style: &str, dot_path: Option<&str>) -> Result<(), YerbaError> {
    let scope_path = dot_path.unwrap_or("");

    if scope_path.contains("[]") {
      let concrete_selectors = self.resolve_selectors(scope_path);

      for selector in concrete_selectors.iter().rev() {
        self.enforce_sequence_indent(style, Some(selector))?;
      }

      return Ok(());
    }

    let scope_node = if scope_path.is_empty() {
      self.root.clone()
    } else {
      match self.navigate(scope_path) {
        Ok(node) => node,
        Err(_) => return Ok(()),
      }
    };

    let has_mismatches = scope_node.descendants().any(|descendant| {
      if !BlockSeq::can_cast(descendant.kind()) {
        return false;
      }

      let sequence = match BlockSeq::cast(descendant.clone()) {
        Some(sequence) => sequence,
        None => return false,
      };

      let first_entry = match sequence.entries().next() {
        Some(entry) => entry,
        None => return false,
      };

      let parent_entry = match descendant.ancestors().find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY) {
        Some(entry) => entry,
        None => return false,
      };

      let key_indent_length = preceding_whitespace_indent(&parent_entry).len();
      let entry_indent_length = preceding_whitespace_indent(first_entry.syntax()).len();
      let is_indented = entry_indent_length > key_indent_length;

      match style {
        "indented" => !is_indented,
        "compact" => is_indented,
        _ => false,
      }
    });

    if !has_mismatches {
      return Ok(());
    }

    loop {
      let source = self.source_text();

      let scope_node = if scope_path.is_empty() {
        self.root.clone()
      } else {
        match self.navigate(scope_path) {
          Ok(node) => node,
          Err(_) => return Ok(()),
        }
      };

      let mut mismatch_ranges: Vec<TextRange> = Vec::new();

      for descendant in scope_node.descendants() {
        if !BlockSeq::can_cast(descendant.kind()) {
          continue;
        }

        let parent_entry = match descendant.ancestors().find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY) {
          Some(entry) => entry,
          None => continue,
        };

        let first_entry = match BlockSeq::cast(descendant.clone()).and_then(|sequence| sequence.entries().next()) {
          Some(entry) => entry,
          None => continue,
        };

        let key_indent = preceding_whitespace_indent(&parent_entry).len();
        let entry_indent = preceding_whitespace_indent(first_entry.syntax()).len();
        let is_indented = entry_indent > key_indent;

        if (style == "indented" && !is_indented) || (style == "compact" && is_indented) {
          mismatch_ranges.push(descendant.text_range());
        }
      }

      if mismatch_ranges.is_empty() {
        return Ok(());
      }

      let mut edits: Vec<(TextRange, String)> = Vec::new();

      for range in &mismatch_ranges {
        if mismatch_ranges.iter().any(|other| other != range && range.contains_range(*other)) {
          continue;
        }

        let node = match scope_node
          .descendants()
          .find(|descendant| BlockSeq::can_cast(descendant.kind()) && descendant.text_range() == *range)
        {
          Some(node) => node,
          None => continue,
        };
        let parent_entry = match node.ancestors().find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY) {
          Some(entry) => entry,
          None => continue,
        };
        let first_entry = match BlockSeq::cast(node).and_then(|sequence| sequence.entries().next()) {
          Some(entry) => entry,
          None => continue,
        };

        let key_indent = preceding_whitespace_indent(&parent_entry).len();
        let entry_indent = preceding_whitespace_indent(first_entry.syntax()).len();

        let indent_diff: i32 = match style {
          "indented" => (key_indent as i32 + 2) - entry_indent as i32,
          "compact" => key_indent as i32 - entry_indent as i32,
          _ => continue,
        };

        if indent_diff == 0 {
          continue;
        }

        let sequence_start: usize = range.start().into();
        let line_start = line_start_at(&source, sequence_start);
        let full_text = &source[line_start..usize::from(range.end())];

        let reindented: String = full_text
          .lines()
          .map(|line| {
            if line.trim().is_empty() {
              String::new()
            } else {
              let current_indent = line.len() - line.trim_start().len();
              let new_indent = (current_indent as i32 + indent_diff).max(0) as usize;
              format!("{}{}", " ".repeat(new_indent), line.trim_start())
            }
          })
          .collect::<Vec<_>>()
          .join("\n");

        edits.push((TextRange::new(rowan::TextSize::from(line_start as u32), range.end()), reindented));
      }

      if edits.is_empty() {
        return Ok(());
      }

      self.apply_edits(edits)?;
    }
  }

  pub fn set_sequence_indent(&mut self, dot_path: &str, style: &str) -> Result<(), YerbaError> {
    let current_style = self
      .get_sequence_indent(dot_path)
      .ok_or_else(|| YerbaError::ParseError(format!("'{}' is not a block sequence", dot_path)))?;

    if current_style == style {
      return Ok(());
    }

    if style != "indented" && style != "compact" {
      return Err(YerbaError::ParseError(format!(
        "unknown sequence indent style '{}'. Valid options: compact, indented",
        style
      )));
    }

    let current_node = self.navigate(dot_path)?;
    let source = self.source_text();

    let entry_node = current_node
      .ancestors()
      .find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY)
      .ok_or_else(|| YerbaError::ParseError("could not find parent map entry".to_string()))?;

    let entry_start: usize = entry_node.text_range().start().into();
    let key_indent_length = column_at(&source, entry_start);

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::ParseError("could not find block sequence".to_string()))?;

    let first_entry = sequence.entries().next();

    if first_entry.is_none() {
      return Ok(());
    }

    let current_entry_indent_length = preceding_whitespace_indent(first_entry.unwrap().syntax()).len();

    let indent_diff: i32 = match style {
      "indented" => (key_indent_length as i32 + 2) - current_entry_indent_length as i32,
      "compact" => key_indent_length as i32 - current_entry_indent_length as i32,
      _ => unreachable!(),
    };

    if indent_diff == 0 {
      return Ok(());
    }

    let sequence_range = sequence.syntax().text_range();
    let sequence_start: usize = sequence_range.start().into();
    let sequence_end: usize = sequence_range.end().into();

    let line_start_of_first_entry = line_start_at(&source, sequence_start);
    let full_text = &source[line_start_of_first_entry..sequence_end];

    let reindented: String = full_text
      .lines()
      .map(|line| {
        if line.trim().is_empty() {
          String::new()
        } else {
          let current_line_indent = line.len() - line.trim_start().len();
          let new_indent = (current_line_indent as i32 + indent_diff).max(0) as usize;
          format!("{}{}", " ".repeat(new_indent), line.trim_start())
        }
      })
      .collect::<Vec<_>>()
      .join("\n");

    let replace_range = TextRange::new(rowan::TextSize::from(line_start_of_first_entry as u32), sequence_range.end());

    self.apply_edit(replace_range, &reindented)
  }

  pub fn set_collection_style(&mut self, dot_path: &str, style: &str) -> Result<(), YerbaError> {
    let current_style = self
      .get_collection_style(dot_path)
      .ok_or_else(|| YerbaError::ParseError(format!("'{}' is not a collection (sequence or map)", dot_path)))?;

    if current_style == style {
      return Ok(());
    }

    let current_node = self.navigate(dot_path)?;
    let value = node_to_yaml_value(&current_node);

    match &value {
      yaml_serde::Value::Sequence(sequence) if sequence.is_empty() => return Ok(()),
      yaml_serde::Value::Mapping(mapping) if mapping.is_empty() => return Ok(()),
      _ => {}
    }

    let source = self.source_text();

    let entry_node = current_node.ancestors().find(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_ENTRY);

    let (key_indent, colon_in_line) = if let Some(ref entry) = entry_node {
      let entry_start: usize = entry.text_range().start().into();
      let indent = column_at(&source, entry_start);

      let entry_text = entry.text().to_string();
      let colon_offset = entry_text.find(':').map(|offset| entry_start + offset);

      (indent, colon_offset)
    } else {
      (0, None)
    };

    match style {
      "flow" => {
        let flow_text = crate::yaml_writer::yaml_value_to_flow_text(&value);

        let collection_node = current_node
          .descendants()
          .find(|descendant| descendant.kind() == SyntaxKind::BLOCK_SEQ || descendant.kind() == SyntaxKind::BLOCK_MAP)
          .ok_or_else(|| YerbaError::ParseError("could not find block collection node".to_string()))?;

        let block_end: usize = collection_node.text_range().end().into();

        if let Some(colon_position) = colon_in_line {
          let replace_range = TextRange::new(rowan::TextSize::from((colon_position + 1) as u32), rowan::TextSize::from(block_end as u32));

          self.apply_edit(replace_range, &format!(" {}", flow_text))
        } else {
          let replace_range = collection_node.text_range();

          self.apply_edit(replace_range, &flow_text)
        }
      }

      "block" => {
        let flow_node = current_node
          .descendants()
          .find(|descendant| descendant.kind() == SyntaxKind::FLOW_SEQ || descendant.kind() == SyntaxKind::FLOW_MAP)
          .ok_or_else(|| YerbaError::ParseError("could not find flow collection node".to_string()))?;

        let flow_end: usize = flow_node.text_range().end().into();
        let value_indent = key_indent + 2;
        let block_text = crate::yaml_writer::yaml_value_to_block_text(&value, value_indent);

        if let Some(colon_position) = colon_in_line {
          let replace_range = TextRange::new(rowan::TextSize::from((colon_position + 1) as u32), rowan::TextSize::from(flow_end as u32));

          self.apply_edit(replace_range, &format!("\n{}", block_text))
        } else {
          let replace_range = flow_node.text_range();

          self.apply_edit(replace_range, &format!("\n{}", block_text))
        }
      }

      _ => Err(YerbaError::ParseError(format!(
        "unknown collection style '{}'. Valid options: flow, block",
        style
      ))),
    }
  }

  pub fn enforce_blank_lines(&mut self, dot_path: &str, blank_lines: usize) -> Result<(), YerbaError> {
    let nodes = if dot_path.contains('[') {
      self.navigate_all_compact(dot_path)
    } else {
      vec![self.navigate(dot_path)?]
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for current_node in &nodes {
      let entry_nodes: Vec<SyntaxNode> = match first_collection(current_node) {
        Some(FirstCollection::Sequence(sequence)) => sequence.entries().map(|entry| entry.syntax().clone()).collect(),
        Some(FirstCollection::Map(map)) => map.entries().map(|entry| entry.syntax().clone()).collect(),
        None => continue,
      };

      for entry_node in entry_nodes.iter().skip(1) {
        collect_blank_line_edits(entry_node, blank_lines, &mut edits);
      }
    }

    self.apply_edits(edits)
  }

  pub fn has_directives_marker(&self) -> bool {
    self.root.descendants_with_tokens().any(|element| element.kind() == SyntaxKind::DIRECTIVES_END)
  }

  pub fn directive_locations(&self) -> Vec<(usize, usize)> {
    let source = self.source_text();

    self
      .root
      .descendants_with_tokens()
      .filter(|element| element.kind() == SyntaxKind::DIRECTIVES_END)
      .map(|element| {
        let offset: usize = element.text_range().start().into();

        (line_at(&source, offset), column_at(&source, offset) + 1)
      })
      .collect()
  }

  pub fn ensure_directives(&mut self) -> Result<(), YerbaError> {
    if self.has_directives_marker() {
      return Ok(());
    }

    let source = self.source_text();
    let new_source = format!("---\n{}", source);

    self.reparse(&new_source)
  }

  pub fn remove_directives(&mut self) -> Result<(), YerbaError> {
    if !self.has_directives_marker() {
      return Ok(());
    }

    let source = self.source_text();
    let new_source = source
      .strip_prefix("---\n")
      .or_else(|| source.strip_prefix("---"))
      .unwrap_or(&source)
      .to_string();

    self.reparse(&new_source)
  }

  pub fn enforce_final_newline(&mut self, count: usize) -> Result<(), YerbaError> {
    let source = self.to_string();
    let trimmed = source.trim_end_matches('\n');
    let expected = format!("{}{}", trimmed, "\n".repeat(count));

    if source != expected {
      self.reparse(&expected)
    } else {
      Ok(())
    }
  }

  pub fn enforce_key_style(&mut self, style: &crate::KeyStyle, dot_path: Option<&str>) -> Result<(), YerbaError> {
    let scope_ranges: Vec<TextRange> = match dot_path {
      Some(path) if !path.is_empty() => self.navigate_all_compact(path).iter().map(|node| node.text_range()).collect(),
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

        let raw_value = match raw_scalar_value(&token) {
          Some(value) => value,
          None => continue,
        };

        let new_text = format_scalar_value(&raw_value, target_kind);

        if new_text != token.text() {
          edits.push((token.text_range(), new_text));
        }
      }
    }

    self.apply_edits(edits)
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
    let source = self.source_text();

    let scope_ranges: Vec<TextRange> = match dot_path {
      Some(path) if !path.is_empty() => self.navigate_all_compact(path).iter().map(|node| node.text_range()).collect(),
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
                    let line = line_at(&source, offset);

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
                    let line = line_at(&source, offset);
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
          let raw_value = match raw_scalar_value(&token) {
            Some(value) => value,
            None => continue,
          };

          if is_yaml_non_string(&raw_value) {
            continue;
          }

          let offset: usize = token.text_range().start().into();
          let line_prefix = &source[line_start_at(&source, offset)..offset];
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

        let raw_value = match raw_scalar_value(&token) {
          Some(value) => value,
          None => continue,
        };

        if current_kind == SyntaxKind::PLAIN_SCALAR && is_yaml_non_string(&raw_value) {
          continue;
        }

        let new_text = match inline_quote_replacement(&raw_value, current_kind, style) {
          Some(new_text) => new_text,
          None => continue,
        };

        if new_text != token.text() {
          edits.push((token.text_range(), new_text));
        }
      }
    }

    self.apply_edits(edits)?;

    Ok(warnings)
  }
}

fn inline_quote_replacement(raw_value: &str, current_kind: SyntaxKind, style: &QuoteStyle) -> Option<String> {
  match style {
    QuoteStyle::Double => {
      if raw_value.contains('"') && current_kind == SyntaxKind::SINGLE_QUOTED_SCALAR {
        return None;
      }

      Some(format_scalar_value(raw_value, SyntaxKind::DOUBLE_QUOTED_SCALAR))
    }

    QuoteStyle::Single => Some(format_scalar_value(raw_value, SyntaxKind::SINGLE_QUOTED_SCALAR)),

    QuoteStyle::Plain => {
      if raw_value.contains('"') || raw_value.contains('\'') || raw_value.contains(':') || raw_value.contains('#') {
        return None;
      }

      Some(raw_value.to_string())
    }

    _ => None,
  }
}
