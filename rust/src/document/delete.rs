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

    let selector = crate::selector::Selector::parse(dot_path);
    let segments = selector.segments();
    let last_segment = segments.last().ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;
    let parent_path = selector.parent_path();

    match last_segment {
      crate::selector::SelectorSegment::Key(last_key) => {
        let has_wildcard = selector.has_wildcard() || selector.has_brackets();

        if parent_path.is_empty() && !has_wildcard {
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

        if parent_nodes.len() == 1 && !has_wildcard {
          let map = find_block_map(&parent_nodes[0]).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

          let entry = find_entry_by_key(&map, last_key).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

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
      crate::selector::SelectorSegment::AllItems => Err(YerbaError::SelectorNotFound(dot_path.to_string())),
    }
  }

  pub fn remove(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

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

    self.remove_node(entries[index].syntax())
  }
}
