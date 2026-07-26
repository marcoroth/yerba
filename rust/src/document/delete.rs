use super::*;

impl Document {
  pub fn rename(&mut self, source_path: &str, destination_path: &str) -> Result<(), YerbaError> {
    Self::validate_path(source_path)?;
    Self::validate_path(destination_path)?;

    let source_parent = source_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");
    let destination_parent = destination_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");
    let destination_key = destination_path.rsplit_once('.').map(|(_, key)| key).unwrap_or(destination_path);

    if source_parent == destination_parent {
      let (parent_path, source_key) = source_path.rsplit_once('.').unwrap_or(("", source_path));

      let parent_node = self.navigate(parent_path)?;
      let map = find_block_map(&parent_node).ok_or_else(|| YerbaError::SelectorNotFound(source_path.to_string()))?;

      let entry = find_entry_by_key(&map, source_key).ok_or_else(|| YerbaError::SelectorNotFound(source_path.to_string()))?;
      let key_node = entry.key().ok_or_else(|| YerbaError::SelectorNotFound(source_path.to_string()))?;
      let key_token = find_scalar_token(key_node.syntax()).ok_or_else(|| YerbaError::SelectorNotFound(source_path.to_string()))?;
      let new_text = format_scalar_value(destination_key, key_token.kind());

      self.replace_token(&key_token, &new_text)
    } else {
      let value = self.get(source_path).ok_or_else(|| YerbaError::SelectorNotFound(source_path.to_string()))?;

      self.delete(source_path)?;
      self.insert_into(destination_path, &value, InsertPosition::Last)
    }
  }

  pub fn delete(&mut self, dot_path: &str) -> Result<(), YerbaError> {
    Self::validate_path(dot_path)?;
    self.refuse_flow_target(dot_path)?;

    let selector = crate::selector::Selector::parse(dot_path);
    let segments = selector.segments();
    let last_segment = segments.last().ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;
    let parent_path = selector.parent_path();

    match last_segment {
      crate::selector::SelectorSegment::Key(last_key) => {
        let has_wildcard = selector.has_wildcard();
        let has_brackets = selector.has_brackets();

        if parent_path.is_empty() && (!has_wildcard || !has_brackets) {
          let parent_node = self.navigate(&parent_path)?;
          let map = find_block_map(&parent_node).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

          let entry = find_entry_by_key(&map, last_key).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

          return self.remove_map_entry(&entry);
        }

        let parent_nodes = self.navigate_all_compact(&parent_path);

        if parent_nodes.is_empty() {
          if has_wildcard {
            return Ok(());
          }

          return Err(YerbaError::SelectorNotFound(dot_path.to_string()));
        }

        if parent_nodes.len() == 1 && (!has_wildcard || !has_brackets) {
          let map = find_block_map(&parent_nodes[0]).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;
          let entry = find_entry_by_key(&map, last_key).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

          if map.entries().count() == 1 {
            return self.collapse_to_empty_flow_map(&parent_nodes[0]);
          }

          return self.remove_map_entry(&entry);
        }

        let mut ranges: Vec<TextRange> = Vec::new();

        for parent_node in &parent_nodes {
          if let Some(map) = find_block_map(parent_node) {
            if let Some(entry) = find_entry_by_key(&map, last_key) {
              ranges.push(removal_range(entry.syntax()));
            }
          }
        }

        let edits = ranges.into_iter().map(|range| (range, String::new())).collect();

        self.apply_edits(edits)
      }

      crate::selector::SelectorSegment::Index(index) => self.remove_at(&parent_path, *index),
      crate::selector::SelectorSegment::AllItems | crate::selector::SelectorSegment::AllKeys => Err(YerbaError::SelectorNotFound(dot_path.to_string())),
    }
  }

  pub fn remove(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    if find_flow_sequence(&current_node).is_some() {
      return Err(YerbaError::FlowCollectionNotWritable(dot_path.to_string()));
    }

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let target_entry = sequence
      .entries()
      .find(|entry| {
        entry
          .flow()
          .and_then(|flow| extract_scalar_text(flow.syntax()))
          .map(|text| text == value)
          .unwrap_or(false)
      })
      .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} item '{}'", dot_path, value)))?;

    if sequence.entries().count() == 1 {
      return self.collapse_to_empty_flow_sequence(&current_node);
    }

    self.remove_node(target_entry.syntax())
  }

  pub fn remove_at(&mut self, dot_path: &str, index: usize) -> Result<(), YerbaError> {
    Self::validate_path(dot_path)?;

    let current_node = self.navigate(dot_path)?;

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    if index >= entries.len() {
      return Err(YerbaError::IndexOutOfBounds(index, entries.len()));
    }

    if entries.len() == 1 {
      return self.collapse_to_empty_flow_sequence(&current_node);
    }

    self.remove_node(entries[index].syntax())
  }

  fn collapse_to_empty_flow_map(&mut self, current_node: &SyntaxNode) -> Result<(), YerbaError> {
    let map_value = current_node
      .descendants()
      .find(|descendant| descendant.kind() == SyntaxKind::BLOCK_MAP_VALUE)
      .ok_or_else(|| YerbaError::SelectorNotFound("Expected a map value".to_string()))?;

    let mut range = map_value.text_range();

    if let Some(previous) = map_value.prev_sibling_or_token().and_then(|element| element.into_token()) {
      if previous.kind() == SyntaxKind::WHITESPACE && previous.text().contains('\n') {
        range = TextRange::new(previous.text_range().start(), range.end());
      }
    }

    self.apply_edit(range, " {}")
  }

  fn collapse_to_empty_flow_sequence(&mut self, current_node: &SyntaxNode) -> Result<(), YerbaError> {
    if current_node.kind() == SyntaxKind::BLOCK_MAP_VALUE {
      let mut range = current_node.text_range();
      let mut comment_text: Option<String> = None;
      let mut sibling = current_node.prev_sibling_or_token();

      while let Some(ref element) = sibling {
        match element {
          rowan::NodeOrToken::Token(token) => {
            if token.kind() == SyntaxKind::COMMENT {
              comment_text = Some(token.text().to_string());
              range = TextRange::new(token.text_range().start(), range.end());
              sibling = token.prev_sibling_or_token();
            } else if token.kind() == SyntaxKind::WHITESPACE {
              range = TextRange::new(token.text_range().start(), range.end());
              sibling = token.prev_sibling_or_token();
            } else {
              break;
            }
          }
          _ => break,
        }
      }

      let replacement = match comment_text {
        Some(comment) => format!(" [] {}", comment),
        None => " []".to_string(),
      };

      return self.apply_edit(range, &replacement);
    }

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence("Expected a sequence".to_string()))?;

    self.apply_edit(removal_range(sequence.syntax()), "[]")
  }
}
