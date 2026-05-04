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

    if let Some(serde_yaml::Value::Sequence(sequence)) = self.get_value(dot_path).as_ref() {
      if let Some(serde_yaml::Value::Mapping(map)) = sequence.first() {
        if let Some((serde_yaml::Value::String(key_name), _)) = map.iter().next() {
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

  pub fn insert_into(&mut self, dot_path: &str, value: &str, position: InsertPosition) -> Result<(), YerbaError> {
    Self::validate_path(dot_path)?;

    if let Ok(current_node) = self.navigate(dot_path) {
      if current_node.descendants().find_map(BlockSeq::cast).is_some() {
        return self.insert_sequence_item(dot_path, value, position);
      }
    }

    let (parent_path, key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));

    self.insert_map_key(parent_path, key, value, position)
  }

  fn insert_sequence_item(&mut self, dot_path: &str, value: &str, position: InsertPosition) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    if entries.is_empty() {
      return Err(YerbaError::SelectorNotFound(dot_path.to_string()));
    }

    let indent = entries
      .get(1)
      .or(entries.first())
      .map(|entry| preceding_whitespace_indent(entry.syntax()))
      .unwrap_or_default();

    let new_item = if value.contains('\n') {
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
    };

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

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

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

    let new_entry_text = format!("{}: {}", key, value);

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
}
