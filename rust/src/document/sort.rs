use super::*;

impl Document {
  pub fn move_item(&mut self, dot_path: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let current_node = self.navigate(dot_path)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    self.reorder_entries(sequence.syntax(), &entries, from, to)
  }

  pub fn move_key(&mut self, dot_path: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let current_node = self.navigate(dot_path)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    let entries: Vec<_> = map.entries().collect();

    self.reorder_entries(map.syntax(), &entries, from, to)
  }

  pub fn resolve_key_index(&self, dot_path: &str, reference: &str) -> Result<usize, YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    if let Ok(index) = reference.parse::<usize>() {
      let length = map.entries().count();

      if index >= length {
        return Err(YerbaError::IndexOutOfBounds(index, length));
      }

      return Ok(index);
    }

    map
      .entries()
      .enumerate()
      .find(|(_index, entry)| {
        entry
          .key()
          .and_then(|key_node| extract_scalar_text(key_node.syntax()))
          .map(|key_text| key_text == reference)
          .unwrap_or(false)
      })
      .map(|(index, _entry)| index)
      .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} key '{}'", dot_path, reference)))
  }

  pub fn resolve_sequence_index(&self, dot_path: &str, reference: &str) -> Result<usize, YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let sequence = current_node
      .descendants()
      .find_map(BlockSeq::cast)
      .ok_or_else(|| YerbaError::NotASequence(dot_path.to_string()))?;

    if let Ok(index) = reference.parse::<usize>() {
      let length = sequence.entries().count();

      if index >= length {
        return Err(YerbaError::IndexOutOfBounds(index, length));
      }

      return Ok(index);
    }

    if crate::selector::Selector::parse(reference).is_relative() {
      return sequence
        .entries()
        .enumerate()
        .find(|(_index, entry)| self.evaluate_condition_on_node(entry.syntax(), reference))
        .map(|(index, _entry)| index)
        .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} condition '{}'", dot_path, reference)));
    }

    sequence
      .entries()
      .enumerate()
      .find(|(_index, entry)| {
        entry
          .flow()
          .and_then(|flow| extract_scalar_text(flow.syntax()))
          .map(|text| text == reference)
          .unwrap_or(false)
      })
      .map(|(index, _entry)| index)
      .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} item '{}'", dot_path, reference)))
  }

  pub fn validate_sort_keys(&self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    if dot_path == "[]" || dot_path.ends_with(".[]") {
      let sequence_path = if dot_path == "[]" { "" } else { &dot_path[..dot_path.len() - 3] };

      return self.validate_each_sort_keys(sequence_path, key_order);
    }

    let current_node = self.navigate(dot_path)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    let unknown_keys: Vec<String> = map
      .entries()
      .filter_map(|entry| entry.key().and_then(|key_node| extract_scalar_text(key_node.syntax())))
      .filter(|key_name| !key_order.contains(&key_name.as_str()))
      .collect();

    if unknown_keys.is_empty() {
      Ok(())
    } else {
      Err(YerbaError::UnknownKeys(unknown_keys))
    }
  }

  pub fn sort_keys(&mut self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    if dot_path == "[]" || dot_path.ends_with(".[]") {
      let sequence_path = if dot_path == "[]" { "" } else { &dot_path[..dot_path.len() - 3] };

      return self.sort_each_keys(sequence_path, key_order);
    }

    let current_node = self.navigate(dot_path)?;

    let map = current_node
      .descendants()
      .find_map(BlockMap::cast)
      .ok_or_else(|| YerbaError::SelectorNotFound(dot_path.to_string()))?;

    let entries: Vec<_> = map.entries().collect();

    if entries.len() <= 1 {
      return Ok(());
    }

    let (groups, range) = collect_groups_with_range(map.syntax());

    let mut keyed: Vec<(String, EntryGroup)> = entries
      .iter()
      .zip(groups)
      .map(|(entry, group)| {
        let key_name = entry.key().and_then(|key_node| extract_scalar_text(key_node.syntax())).unwrap_or_default();
        (key_name, group)
      })
      .collect();

    let original_keys: Vec<String> = keyed.iter().map(|(key, _)| key.clone()).collect();

    keyed.sort_by(|(key_a, _), (key_b, _)| {
      let position_a = key_order.iter().position(|&key| key == key_a);
      let position_b = key_order.iter().position(|&key| key == key_b);

      match (position_a, position_b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => {
          let original_a = original_keys.iter().position(|key| key == key_a).unwrap();
          let original_b = original_keys.iter().position(|key| key == key_b).unwrap();
          original_a.cmp(&original_b)
        }
      }
    });

    let sorted_keys: Vec<&str> = keyed.iter().map(|(key, _)| key.as_str()).collect();
    let orig_refs: Vec<&str> = original_keys.iter().map(|key| key.as_str()).collect();

    if sorted_keys == orig_refs {
      return Ok(());
    }

    let indent = entries.get(1).map(|entry| preceding_whitespace_indent(entry.syntax())).unwrap_or_default();

    let sorted_groups: Vec<EntryGroup> = keyed.into_iter().map(|(_, group)| group).collect();
    let map_text = rebuild_from_groups(&sorted_groups, &indent, false);

    self.apply_edit(range, &map_text)
  }

  pub fn sort_each_keys(&mut self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for entry in sequence.entries() {
      let entry_node = entry.syntax();

      let map = match entry_node.descendants().find_map(BlockMap::cast) {
        Some(map) => map,
        None => continue,
      };

      let entries: Vec<_> = map.entries().collect();

      if entries.len() <= 1 {
        continue;
      }

      let (groups, group_range) = collect_groups_with_range(map.syntax());

      let mut keyed: Vec<(String, EntryGroup)> = entries
        .iter()
        .zip(groups)
        .map(|(entry, group)| {
          let key_name = entry.key().and_then(|key_node| extract_scalar_text(key_node.syntax())).unwrap_or_default();
          (key_name, group)
        })
        .collect();

      let original_keys: Vec<String> = keyed.iter().map(|(key, _)| key.clone()).collect();

      keyed.sort_by(|(key_a, _), (key_b, _)| {
        let position_a = key_order.iter().position(|&key| key == key_a);
        let position_b = key_order.iter().position(|&key| key == key_b);

        match (position_a, position_b) {
          (Some(a), Some(b)) => a.cmp(&b),
          (Some(_), None) => std::cmp::Ordering::Less,
          (None, Some(_)) => std::cmp::Ordering::Greater,

          (None, None) => {
            let original_a = original_keys.iter().position(|key| key == key_a).unwrap();
            let original_b = original_keys.iter().position(|key| key == key_b).unwrap();

            original_a.cmp(&original_b)
          }
        }
      });

      let sorted_keys: Vec<&str> = keyed.iter().map(|(key, _)| key.as_str()).collect();
      let orig_refs: Vec<&str> = original_keys.iter().map(|key| key.as_str()).collect();

      if sorted_keys == orig_refs {
        continue;
      }

      let indent = entries.get(1).map(|entry| preceding_whitespace_indent(entry.syntax())).unwrap_or_default();

      let sorted_groups: Vec<EntryGroup> = keyed.into_iter().map(|(_, group)| group).collect();
      let map_text = rebuild_from_groups(&sorted_groups, &indent, false);
      edits.push((group_range, map_text));
    }

    if edits.is_empty() {
      return Ok(());
    }

    edits.reverse();

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

  pub fn validate_each_sort_keys(&self, dot_path: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let current_node = self.navigate(dot_path)?;

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    let mut all_unknown: Vec<String> = Vec::new();

    for entry in sequence.entries() {
      if let Some(map) = entry.syntax().descendants().find_map(BlockMap::cast) {
        for map_entry in map.entries() {
          if let Some(key_name) = map_entry.key().and_then(|key_node| extract_scalar_text(key_node.syntax())) {
            if !key_order.contains(&key_name.as_str()) && !all_unknown.contains(&key_name) {
              all_unknown.push(key_name);
            }
          }
        }
      }
    }

    if all_unknown.is_empty() {
      Ok(())
    } else {
      Err(YerbaError::UnknownKeys(all_unknown))
    }
  }

  pub fn sort_items(&mut self, dot_path: &str, sort_fields: &[SortField], case_sensitive: bool) -> Result<(), YerbaError> {
    if dot_path.contains("[].") {
      return self.sort_each_items(dot_path, sort_fields, case_sensitive);
    }

    let current_node = self.navigate(dot_path)?;

    let sequence = match current_node.descendants().find_map(BlockSeq::cast) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    let entries: Vec<_> = sequence.entries().collect();

    if entries.len() <= 1 {
      return Ok(());
    }

    let (groups, range) = collect_groups_with_range(sequence.syntax());

    let mut sortable: Vec<(Vec<String>, EntryGroup)> = entries
      .iter()
      .zip(groups)
      .map(|(entry, group)| {
        let sort_values = if sort_fields.is_empty() {
          vec![entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())).unwrap_or_default()]
        } else {
          sort_fields
            .iter()
            .map(|field| {
              if field.path.is_empty() {
                entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())).unwrap_or_default()
              } else {
                let nodes = navigate_from_node(entry.syntax(), &field.path);
                nodes.first().and_then(extract_scalar_text).unwrap_or_default()
              }
            })
            .collect()
        };

        (sort_values, group)
      })
      .collect();

    let original_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

    sortable.sort_by(|(values_a, _), (values_b, _)| {
      for (index, field) in sort_fields.iter().enumerate().take(values_a.len()) {
        let value_a = &values_a[index];
        let value_b = &values_b[index];

        let ordering = if case_sensitive {
          value_a.cmp(value_b)
        } else {
          value_a.to_lowercase().cmp(&value_b.to_lowercase())
        };

        let ordering = if field.ascending { ordering } else { ordering.reverse() };

        if ordering != std::cmp::Ordering::Equal {
          return ordering;
        }
      }

      if sort_fields.is_empty() && !values_a.is_empty() && !values_b.is_empty() {
        return if case_sensitive {
          values_a[0].cmp(&values_b[0])
        } else {
          values_a[0].to_lowercase().cmp(&values_b[0].to_lowercase())
        };
      }

      std::cmp::Ordering::Equal
    });

    let sorted_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

    if sorted_bodies == original_bodies {
      return Ok(());
    }

    let indent = entries.get(1).map(|entry| preceding_whitespace_indent(entry.syntax())).unwrap_or_default();

    let sorted_groups: Vec<EntryGroup> = sortable.into_iter().map(|(_, group)| group).collect();
    let sequence_text = rebuild_from_groups(&sorted_groups, &indent, true);

    self.apply_edit(range, &sequence_text)
  }

  fn sort_each_items(&mut self, dot_path: &str, sort_fields: &[SortField], case_sensitive: bool) -> Result<(), YerbaError> {
    let (parent_path, child_path) = if let Some(last_bracket) = dot_path.rfind("[].") {
      (&dot_path[..last_bracket + 2], &dot_path[last_bracket + 3..])
    } else {
      (dot_path, "")
    };

    let parent_nodes = self.navigate_all(parent_path);
    let source = self.root.text().to_string();
    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for parent_node in &parent_nodes {
      let child_nodes = if child_path.is_empty() {
        vec![parent_node.clone()]
      } else {
        navigate_from_node(parent_node, child_path)
      };

      for child_node in &child_nodes {
        let sequence = match child_node.descendants().find_map(BlockSeq::cast) {
          Some(sequence) => sequence,
          None => continue,
        };

        let entries: Vec<_> = sequence.entries().collect();

        if entries.len() <= 1 {
          continue;
        }

        let (groups, group_range) = collect_groups_with_range(sequence.syntax());

        let mut sortable: Vec<(Vec<String>, EntryGroup)> = entries
          .iter()
          .zip(groups)
          .map(|(entry, group)| {
            let sort_values = if sort_fields.is_empty() {
              vec![entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())).unwrap_or_default()]
            } else {
              sort_fields
                .iter()
                .map(|field| {
                  if field.path.is_empty() {
                    entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())).unwrap_or_default()
                  } else {
                    let nodes = navigate_from_node(entry.syntax(), &field.path);
                    nodes.first().and_then(extract_scalar_text).unwrap_or_default()
                  }
                })
                .collect()
            };

            (sort_values, group)
          })
          .collect();

        let original_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

        sortable.sort_by(|(values_a, _), (values_b, _)| {
          for (index, field) in sort_fields.iter().enumerate().take(values_a.len()) {
            let value_a = &values_a[index];
            let value_b = &values_b[index];

            let ordering = if case_sensitive {
              value_a.cmp(value_b)
            } else {
              value_a.to_lowercase().cmp(&value_b.to_lowercase())
            };

            let ordering = if field.ascending { ordering } else { ordering.reverse() };

            if ordering != std::cmp::Ordering::Equal {
              return ordering;
            }
          }

          if sort_fields.is_empty() && !values_a.is_empty() && !values_b.is_empty() {
            return if case_sensitive {
              values_a[0].cmp(&values_b[0])
            } else {
              values_a[0].to_lowercase().cmp(&values_b[0].to_lowercase())
            };
          }

          std::cmp::Ordering::Equal
        });

        let sorted_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

        if sorted_bodies == original_bodies {
          continue;
        }

        let indent = entries.get(1).map(|entry| preceding_whitespace_indent(entry.syntax())).unwrap_or_default();
        let sorted_groups: Vec<EntryGroup> = sortable.into_iter().map(|(_, group)| group).collect();
        let sequence_text = rebuild_from_groups(&sorted_groups, &indent, true);

        edits.push((group_range, sequence_text));
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

  pub fn reorder_items(&mut self, dot_path: &str, by: &str, desired_order: &[&str]) -> Result<(), YerbaError> {
    let field = by.strip_prefix('.').unwrap_or(by);
    let is_scalar = field.is_empty() || field == ".";

    let items_selector = if is_scalar {
      if dot_path.is_empty() {
        "[]".to_string()
      } else {
        format!("{}[]", dot_path)
      }
    } else if dot_path.is_empty() {
      format!("[].{}", field)
    } else {
      format!("{}[].{}", dot_path, field)
    };

    let labels: Vec<String> = match self.get_value(&items_selector) {
      Some(serde_yaml::Value::Sequence(sequence)) => sequence
        .iter()
        .map(|value| match value {
          serde_yaml::Value::String(string) => string.clone(),
          _ => String::new(),
        })
        .collect(),
      _ => return Err(YerbaError::SelectorNotFound(items_selector)),
    };

    let mut used = vec![false; labels.len()];
    let mut moves: Vec<usize> = Vec::new();

    for desired in desired_order {
      let found = labels.iter().enumerate().find(|(index, label)| label.as_str() == *desired && !used[*index]);

      if let Some((index, _)) = found {
        moves.push(index);
        used[index] = true;
      } else {
        return Err(YerbaError::SelectorNotFound(format!("no item found with {} == \"{}\"", by, desired)));
      }
    }

    let missing: Vec<&String> = labels.iter().enumerate().filter(|(index, _)| !used[*index]).map(|(_, label)| label).collect();

    if !missing.is_empty() {
      let missing_list = missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");

      return Err(YerbaError::SelectorNotFound(format!(
        "order must specify all {} items, but {} missing: {}",
        labels.len(),
        missing.len(),
        missing_list
      )));
    }

    for target in 0..moves.len() {
      let source = moves[target];

      if source != target {
        self.move_item(dot_path, source, target)?;

        for item in moves.iter_mut().skip(target + 1) {
          if *item >= target && *item < source {
            *item += 1;
          } else if *item == source {
            *item = target;
          }
        }
      }
    }

    Ok(())
  }
}
