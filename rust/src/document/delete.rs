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

      let map = parent_node
        .descendants()
        .find_map(BlockMap::cast)
        .ok_or_else(|| YerbaError::SelectorNotFound(source_path.to_string()))?;

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

    let (parent_path, last_key) = dot_path.rsplit_once('.').unwrap_or(("", dot_path));
    let parent_node = self.navigate(parent_path)?;

    let map = parent_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    let entry = find_entry_by_key(&map, last_key).ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    self.remove_node(entry.syntax())
  }

  pub fn remove(&mut self, dot_path: &str, value: &str) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

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

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    if index >= entries.len() {
      return Err(YerbaError::IndexOutOfBounds(index, entries.len()));
    }

    self.remove_node(entries[index].syntax())
  }
}
