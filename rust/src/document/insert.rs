use super::*;

impl Document {
  pub fn append(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    self.insert_into(dot_path, value, InsertPosition::Last)
  }

  pub fn detect_sequence_quote_style(&self, dot_path: &str) -> QuoteStyle {
    let try_paths = if dot_path.is_empty() {
      vec!["[]".to_string(), "[0]".to_string()]
    } else {
      vec![format!("{}[]", dot_path), format!("{}[0]", dot_path)]
    };

    for try_path in &try_paths {
      for scalar in self.get_all_typed(try_path) {
        if scalar.kind == SyntaxKind::DOUBLE_QUOTED_SCALAR {
          return QuoteStyle::Double;
        } else if scalar.kind == SyntaxKind::SINGLE_QUOTED_SCALAR {
          return QuoteStyle::Single;
        }
      }
    }

    if let Some(yaml_serde::Value::Sequence(sequence)) = self.get_value(dot_path).as_ref() {
      if let Some(yaml_serde::Value::Mapping(map)) = sequence.first() {
        if let Some((yaml_serde::Value::String(key_name), _)) = map.iter().next() {
          let deep_path = if dot_path.is_empty() {
            format!("[].{}", key_name)
          } else {
            format!("{}[].{}", dot_path, key_name)
          };

          for scalar in self.get_all_typed(&deep_path) {
            if scalar.kind == SyntaxKind::DOUBLE_QUOTED_SCALAR {
              return QuoteStyle::Double;
            } else if scalar.kind == SyntaxKind::SINGLE_QUOTED_SCALAR {
              return QuoteStyle::Single;
            }
          }
        }
      }
    }

    QuoteStyle::Plain
  }

  pub fn insert_object(&mut self, dot_path: &str, json_value: &serde_json::Value, position: InsertPosition) -> Result<(), YerbaError> {
    let quote_style = self.detect_sequence_quote_style(dot_path);
    let yaml_text = crate::yaml_writer::json_to_yaml_text(json_value, &quote_style, 0);

    self.insert_into(dot_path, &yaml_text, position)
  }

  pub fn insert_objects(&mut self, dot_path: &str, json_values: &[serde_json::Value]) -> Result<(), YerbaError> {
    if json_values.is_empty() {
      return Ok(());
    }

    let quote_style = self.detect_sequence_quote_style(dot_path);
    let current_node = self.navigate(dot_path)?;

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    if entries.is_empty() {
      return Err(YerbaError::SelectorNotFound(dot_path.to_string()));
    }

    let indent = entries
      .get(1)
      .or(entries.first())
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let mut new_text = String::new();

    for json_value in json_values {
      let yaml_text = crate::yaml_writer::json_to_yaml_text(json_value, &quote_style, 0);
      let new_item = Self::format_sequence_item(&yaml_text, &indent);

      new_text.push_str(&format!("\n{}{}", indent, new_item));
    }

    let last_entry = entries.last().unwrap();

    self.insert_after_node(last_entry.syntax(), &new_text)
  }

  pub fn insert_into(&mut self, dot_path: &str, value: &str, position: InsertPosition) -> Result<(), YerbaError> {
    Self::validate_path(dot_path)?;
    self.refuse_flow_target(dot_path)?;

    if let Some((parent_path, _)) = dot_path.rsplit_once('.') {
      if let Ok(parent) = self.navigate(parent_path) {
        let occupied_flow = find_flow_map(&parent).map(|map| !flow_map_entries(&map).is_empty()).unwrap_or(false)
          || find_flow_sequence(&parent)
            .map(|sequence| !flow_sequence_entries(&sequence).is_empty())
            .unwrap_or(false);

        if occupied_flow {
          return Err(YerbaError::FlowCollectionNotWritable(dot_path.to_string()));
        }
      }
    }

    if crate::selector::Selector::parse(dot_path)
      .segments()
      .iter()
      .any(|segment| matches!(segment, crate::selector::SelectorSegment::AllKeys))
    {
      return Err(YerbaError::SelectorNotFound(dot_path.to_string()));
    }

    if let Ok(current_node) = self.navigate(dot_path) {
      if find_block_sequence(&current_node).is_some()
        || matches!(self.get_value(dot_path).as_ref(), Some(yaml_serde::Value::Sequence(sequence)) if sequence.is_empty())
      {
        return self.insert_sequence_item(dot_path, value, position);
      }
    }

    let selector = crate::selector::Selector::parse(dot_path);
    let has_wildcard = selector.has_wildcard() || selector.has_brackets();
    let (parent_path, key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));

    if has_wildcard {
      let count = self.navigate_all_compact(parent_path).len();

      if count == 0 {
        return Ok(());
      }

      for i in (0..count).rev() {
        let nodes = self.navigate_all_compact(parent_path);

        let node = match nodes.get(i) {
          Some(node) => node.clone(),
          None => continue,
        };

        let map = match find_block_map(&node) {
          Some(map) => map,
          None => {
            let has_empty_flow_map = node
              .descendants()
              .any(|descendant| descendant.kind() == SyntaxKind::FLOW_MAP && !descendant.descendants().any(|child| child.kind() == SyntaxKind::FLOW_MAP_ENTRY));

            if has_empty_flow_map {
              let selectors = self.resolve_selectors(parent_path);

              if let Some(selector) = selectors.get(i) {
                self.replace_empty_inline_map(selector, key, value)?;
                continue;
              }
            }

            continue;
          }
        };

        if find_entry_by_key(&map, key).is_some() {
          continue;
        }

        let entries: Vec<_> = map.entries().collect();

        if entries.is_empty() {
          continue;
        }

        let first_entry = entries.first().unwrap();

        let start_col = {
          let offset: usize = first_entry.syntax().text_range().start().into();
          let source = self.source_text();

          column_at(&source, offset)
        };

        let indent = " ".repeat(start_col);
        let new_entry_text = Self::format_map_entry(key, value, &indent);

        match &position {
          InsertPosition::After(target_key) => {
            let target = find_entry_by_key(&map, target_key);
            let after_node = target.map(|e| e.syntax().clone()).unwrap_or_else(|| entries.last().unwrap().syntax().clone());
            let new_text = format!("\n{}{}", indent, new_entry_text);

            self.insert_after_node(&after_node, &new_text)?;
          }

          InsertPosition::Before(target_key) => {
            if let Some(target_entry) = find_entry_by_key(&map, target_key) {
              let target_range = target_entry.syntax().text_range();
              let replacement = format!("{}\n{}", new_entry_text, indent);
              let insert_range = TextRange::new(target_range.start(), target_range.start());

              self.apply_edit(insert_range, &replacement)?;
            } else {
              let last_entry = entries.last().unwrap();
              let new_text = format!("\n{}{}", indent, new_entry_text);

              self.insert_after_node(last_entry.syntax(), &new_text)?;
            }
          }

          _ => {
            let last_entry = entries.last().unwrap();
            let new_text = format!("\n{}{}", indent, new_entry_text);

            self.insert_after_node(last_entry.syntax(), &new_text)?;
          }
        }
      }

      return Ok(());
    }

    self.insert_map_key(parent_path, key, value, position)
  }

  fn insert_sequence_item(&mut self, dot_path: &str, value: &str, position: InsertPosition) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let Some(sequence) = find_block_sequence(&current_node) else {
      if matches!(self.get_value(dot_path).as_ref(), Some(yaml_serde::Value::Sequence(sequence)) if sequence.is_empty()) {
        return self.replace_empty_inline_sequence(dot_path, value);
      }

      return Err(YerbaError::NotASequence(dot_path.to_string()));
    };

    let entries: Vec<_> = sequence.entries().collect();

    if entries.is_empty() {
      return Err(YerbaError::SelectorNotFound(dot_path.to_string()));
    }

    let indent = entries
      .get(1)
      .or(entries.first())
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let new_item = Self::format_sequence_item(value, &indent);

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
          .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} item '{}'", dot_path, target_value)))?;

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
          .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} item '{}'", dot_path, target_value)))?;

        let new_text = format!("\n{}{}", indent, new_item);

        self.insert_after_node(target_entry.syntax(), &new_text)
      }

      InsertPosition::BeforeCondition(condition) => {
        let target_entry = entries
          .iter()
          .find(|entry| self.evaluate_condition_on_node(entry.syntax(), &condition))
          .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} condition '{}'", dot_path, condition)))?;

        let target_range = target_entry.syntax().text_range();
        let replacement = format!("{}\n{}", new_item, indent);
        let insert_range = TextRange::new(target_range.start(), target_range.start());

        self.apply_edit(insert_range, &replacement)
      }

      InsertPosition::AfterCondition(condition) => {
        let target_entry = entries
          .iter()
          .find(|entry| self.evaluate_condition_on_node(entry.syntax(), &condition))
          .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} condition '{}'", dot_path, condition)))?;

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

  fn insert_map_key(&mut self, dot_path: &str, key: &str, value: &str, position: InsertPosition) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let map = match find_block_map(&current_node) {
      Some(map) => map,
      None => {
        let flow_map = current_node.descendants().find(|descendant| descendant.kind() == SyntaxKind::FLOW_MAP);

        if let Some(flow_map_node) = flow_map {
          let is_empty = !flow_map_node.descendants().any(|descendant| descendant.kind() == SyntaxKind::FLOW_MAP_ENTRY);

          if is_empty {
            return self.replace_empty_inline_map(dot_path, key, value);
          }
        }

        return Err(YerbaError::SelectorNotFound(dot_path.to_string()));
      }
    };

    let entries: Vec<_> = map.entries().collect();

    if entries.is_empty() {
      let indent = preceding_whitespace_indent(map.syntax());
      let new_entry = format!("\n{}{}: {}", indent, key, value);

      return self.insert_after_node(map.syntax(), &new_entry);
    }

    if find_entry_by_key(&map, key).is_some() {
      return Err(YerbaError::ParseError(format!("key '{}' already exists at '{}'", key, dot_path)));
    }

    let indent = entries
      .get(1)
      .or(entries.first())
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let new_entry_text = Self::format_map_entry(key, value, &indent);

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
        let target_entry = find_entry_by_key(&map, &target_key).ok_or_else(|| YerbaError::SelectorNotFound(format!("{}.{}", dot_path, target_key)))?;

        let target_range = target_entry.syntax().text_range();
        let replacement = format!("{}\n{}", new_entry_text, indent);
        let insert_range = TextRange::new(target_range.start(), target_range.start());

        self.apply_edit(insert_range, &replacement)
      }

      InsertPosition::After(target_key) => {
        let target_entry = find_entry_by_key(&map, &target_key).ok_or_else(|| YerbaError::SelectorNotFound(format!("{}.{}", dot_path, target_key)))?;

        let new_text = format!("\n{}{}", indent, new_entry_text);

        self.insert_after_node(target_entry.syntax(), &new_text)
      }

      InsertPosition::BeforeCondition(_) | InsertPosition::AfterCondition(_) => self.insert_map_key(dot_path, key, value, InsertPosition::Last),

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

  fn replace_empty_inline_map(&mut self, dot_path: &str, key: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    if dot_path.is_empty() {
      let flow_map = current_node
        .descendants()
        .find(|descendant| descendant.kind() == SyntaxKind::FLOW_MAP)
        .ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

      let range = flow_map.text_range();

      let source = self.source_text();
      let start: usize = range.start().into();
      let before = &source[..start];
      let trimmed_length = before.trim_end_matches([' ', '\n']).len();

      let adjusted_range = TextRange::new(rowan::TextSize::from(trimmed_length as u32), range.end());
      let replacement = format!("\n{}: {}", key, value);

      return self.apply_edit(adjusted_range, &replacement);
    }

    let (parent_path, map_key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));
    let parent_node = self.navigate(parent_path)?;

    let map = find_block_map(&parent_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    let entry = find_entry_by_key(&map, map_key).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;
    let entry_indent = preceding_whitespace_indent(entry.syntax());
    let child_indent = format!("{}  ", entry_indent);
    let new_entry_text = format!("{}{}: {}", child_indent, key, value);

    let mut range = current_node.text_range();

    if let Some(previous) = current_node.prev_sibling_or_token().and_then(|element| element.into_token()) {
      if previous.kind() == SyntaxKind::WHITESPACE && !previous.text().contains('\n') {
        range = TextRange::new(previous.text_range().start(), range.end());
      }
    }

    let replacement = format!("\n{}", new_entry_text);

    self.apply_edit(range, &replacement)
  }

  fn replace_empty_inline_sequence(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    if dot_path.is_empty() {
      let current_node = self.navigate(dot_path)?;
      let flow_sequence = current_node
        .descendants()
        .find(|descendant| descendant.kind() == SyntaxKind::FLOW_SEQ)
        .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

      let new_item = Self::format_sequence_item(value, "");
      let range = flow_sequence.text_range();
      let source = self.source_text();
      let start: usize = range.start().into();
      let before = &source[..start];

      let trimmed_length = before.trim_end_matches([' ', '\n']).len();
      let adjusted_start = trimmed_length;

      let adjusted_range = TextRange::new(rowan::TextSize::from(adjusted_start as u32), range.end());
      let replacement = format!("\n{}", new_item);

      return self.apply_edit(adjusted_range, &replacement);
    }

    let (parent_path, key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));
    let parent_node = self.navigate(parent_path)?;

    let map = find_block_map(&parent_node).ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entry = find_entry_by_key(&map, key).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;
    let item_indent = format!("{}  ", preceding_whitespace_indent(entry.syntax()));
    let new_item = Self::format_sequence_item(value, &item_indent);
    let current_node = self.navigate(dot_path)?;
    let inline_comment = self.trailing_inline_comment(&current_node);

    let mut range = current_node.text_range();

    if let Some(previous) = current_node.prev_sibling_or_token().and_then(|element| element.into_token()) {
      if previous.kind() == SyntaxKind::WHITESPACE && !previous.text().contains('\n') {
        range = TextRange::new(previous.text_range().start(), range.end());
      }
    }

    if let Some((_, comment_end)) = inline_comment {
      range = TextRange::new(range.start(), comment_end);
    }

    let replacement = match inline_comment {
      Some((comment_text, _)) => format!(" {}\n{}{}", comment_text, item_indent, new_item),
      None => format!("\n{}{}", item_indent, new_item),
    };

    self.apply_edit(range, &replacement)
  }

  fn trailing_inline_comment(&self, node: &SyntaxNode) -> Option<(String, rowan::TextSize)> {
    let source = self.source_text();
    let start: usize = node.text_range().end().into();
    let rest = &source[start..];
    let line_end = rest.find('\n').unwrap_or(rest.len());
    let trailing = &rest[..line_end];
    let comment_start = trailing.find('#')?;

    if !trailing[..comment_start].trim().is_empty() {
      return None;
    }

    Some((trailing[comment_start..].to_string(), rowan::TextSize::from((start + line_end) as u32)))
  }

  fn format_map_entry(key: &str, value: &str, indent: &str) -> String {
    let is_block_value = value.contains('\n') || value.starts_with("- ");

    if !is_block_value {
      return format!("{}: {}", key, value);
    }

    let value_indent = format!("{}  ", indent);
    let lines: Vec<&str> = value.lines().collect();

    let min_indent = lines
      .iter()
      .filter(|line| !line.trim().is_empty())
      .map(|line| line.len() - line.trim_start().len())
      .min()
      .unwrap_or(0);

    let indented_lines: Vec<String> = lines
      .iter()
      .map(|line| {
        if line.trim().is_empty() {
          String::new()
        } else {
          let relative = &line[min_indent..];
          format!("{}{}", value_indent, relative)
        }
      })
      .collect();

    format!("{}:\n{}", key, indented_lines.join("\n"))
  }

  fn format_sequence_item(value: &str, indent: &str) -> String {
    if value.contains('\n') {
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
    }
  }
}
