use super::*;

impl Document {
  pub fn rename(&mut self, source_selector: &str, destination_selector: &str) -> Result<(), YerbaError> {
    Self::validate_selector(source_selector)?;
    Self::validate_selector(destination_selector)?;

    let source_parent = source_selector.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");
    let destination_parent = destination_selector.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");
    let destination_key = destination_selector.rsplit_once('.').map(|(_, key)| key).unwrap_or(destination_selector);

    if source_parent == destination_parent {
      let (parent_selector, source_key) = source_selector.rsplit_once('.').unwrap_or(("", source_selector));

      let parent_node = self.navigate(parent_selector)?;
      let map = find_block_map(&parent_node).ok_or_else(|| YerbaError::SelectorNotFound(source_selector.to_string()))?;

      let entry = find_entry_by_key(&map, source_key).ok_or_else(|| YerbaError::SelectorNotFound(source_selector.to_string()))?;
      let key_node = entry.key().ok_or_else(|| YerbaError::SelectorNotFound(source_selector.to_string()))?;
      let key_token = find_scalar_token(key_node.syntax()).ok_or_else(|| YerbaError::SelectorNotFound(source_selector.to_string()))?;
      let new_text = format_scalar_value(destination_key, key_token.kind());

      self.replace_token(&key_token, &new_text)
    } else {
      let value = self
        .get(source_selector)
        .ok_or_else(|| YerbaError::SelectorNotFound(source_selector.to_string()))?;

      self.delete(source_selector)?;
      self.insert_into(destination_selector, &value, InsertPosition::Last)
    }
  }

  pub fn delete(&mut self, selector: &str) -> Result<(), YerbaError> {
    Self::validate_selector(selector)?;
    self.refuse_flow_target(selector)?;

    let parsed = crate::selector::Selector::parse(selector);
    let segments = parsed.segments();
    let last_segment = segments.last().ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;
    let parent_selector = parsed.parent_selector();

    match last_segment {
      crate::selector::SelectorSegment::Key(last_key) => {
        let has_wildcard = parsed.has_wildcard();
        let has_brackets = parsed.has_brackets();

        if parent_selector.is_empty() && (!has_wildcard || !has_brackets) {
          let parent_node = self.navigate(&parent_selector)?;
          let map = find_block_map(&parent_node).ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;

          let entry = find_entry_by_key(&map, last_key).ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;

          return self.remove_map_entry(&entry);
        }

        let parent_nodes = self.navigate_all_compact(&parent_selector);

        if parent_nodes.is_empty() {
          if has_wildcard {
            return Ok(());
          }

          return Err(YerbaError::SelectorNotFound(selector.to_string()));
        }

        if parent_nodes.len() == 1 && (!has_wildcard || !has_brackets) {
          let map = find_block_map(&parent_nodes[0]).ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;
          let entry = find_entry_by_key(&map, last_key).ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;

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

      crate::selector::SelectorSegment::Index(index) => self.remove_at(&parent_selector, *index),
      crate::selector::SelectorSegment::IndexFromEnd(_) => {
        let parent_node = self.navigate(&parent_selector)?;
        let length = find_block_sequence(&parent_node)
          .map(|sequence| sequence.entries().count())
          .or_else(|| find_flow_sequence(&parent_node).map(|sequence| flow_sequence_entries(&sequence).len()))
          .ok_or_else(|| YerbaError::NotASequence(parent_selector.clone()))?;
        let index = last_segment
          .sequence_index(length)
          .ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;

        self.remove_at(&parent_selector, index)
      }
      crate::selector::SelectorSegment::AllItems | crate::selector::SelectorSegment::AllKeys => Err(YerbaError::SelectorNotFound(selector.to_string())),
    }
  }

  pub fn remove(&mut self, selector: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(selector)?;

    if find_flow_sequence(&current_node).is_some() {
      return Err(YerbaError::FlowCollectionNotWritable(selector.to_string()));
    }

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::NotASequence(selector.to_string()))?;

    let target_entry = sequence
      .entries()
      .find(|entry| {
        entry
          .flow()
          .and_then(|flow| extract_scalar_text(flow.syntax()))
          .map(|text| text == value)
          .unwrap_or(false)
      })
      .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} item '{}'", selector, value)))?;

    if sequence.entries().count() == 1 {
      return self.collapse_to_empty_flow_sequence(&current_node);
    }

    self.remove_node(target_entry.syntax())
  }

  pub fn remove_at(&mut self, selector: &str, index: usize) -> Result<(), YerbaError> {
    Self::validate_selector(selector)?;

    let current_node = self.navigate(selector)?;

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::NotASequence(selector.to_string()))?;

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
