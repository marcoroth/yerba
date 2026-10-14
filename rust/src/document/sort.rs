use super::*;

impl Document {
  pub fn move_item(&mut self, selector: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let current_node = self.navigate(selector)?;

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::NotASequence(selector.to_string()))?;

    let entries: Vec<_> = sequence.entries().collect();

    self.reorder_entries(sequence.syntax(), &entries, from, to)
  }

  pub fn move_key(&mut self, selector: &str, from: usize, to: usize) -> Result<(), YerbaError> {
    if from == to {
      return Ok(());
    }

    let current_node = self.navigate(selector)?;

    let map = find_block_map(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;

    let entries: Vec<_> = map.entries().collect();

    self.reorder_entries(map.syntax(), &entries, from, to)
  }

  pub fn resolve_key_index(&self, selector: &str, reference: &str) -> Result<usize, YerbaError> {
    let current_node = self.navigate(selector)?;

    let map = find_block_map(&current_node).ok_or_else(|| YerbaError::SelectorNotFound(selector.to_string()))?;

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
      .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} key '{}'", selector, reference)))
  }

  pub fn resolve_sequence_index(&self, selector: &str, reference: &str) -> Result<usize, YerbaError> {
    let current_node = self.navigate(selector)?;

    let sequence = find_block_sequence(&current_node).ok_or_else(|| YerbaError::NotASequence(selector.to_string()))?;

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
        .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} condition '{}'", selector, reference)));
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
      .ok_or_else(|| YerbaError::SelectorNotFound(format!("{} item '{}'", selector, reference)))
  }

  pub fn validate_sort_keys(&self, selector: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    if let Some(sequence_selector) = strip_bracket_suffix(selector) {
      return self.validate_each_sort_keys(sequence_selector, key_order);
    }

    let current_node = match self.navigate(selector) {
      Ok(node) => node,
      Err(_) => return Ok(()),
    };

    let map = match find_block_map(&current_node) {
      Some(map) => map,
      None => return Ok(()),
    };

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

  pub fn sort_keys(&mut self, selector: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    if let Some(sequence_selector) = strip_bracket_suffix(selector) {
      return self.sort_each_keys(sequence_selector, key_order);
    }

    let current_node = match self.navigate(selector) {
      Ok(node) => node,
      Err(_) => return Ok(()),
    };

    let map = match find_block_map(&current_node) {
      Some(map) => map,
      None => return Ok(()),
    };

    match sort_map_edit(&map, key_order) {
      Some(edit) => self.apply_edits(vec![edit]),
      None => Ok(()),
    }
  }

  pub fn sort_each_keys(&mut self, selector: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let nodes = if selector.is_empty() {
      match self.navigate(selector) {
        Ok(node) => vec![node],
        Err(_) => return Ok(()),
      }
    } else {
      let found = self.navigate_all_compact(selector);
      if found.is_empty() {
        return Ok(());
      }
      found
    };

    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for current_node in &nodes {
      let sequence = match find_block_sequence(current_node) {
        Some(sequence) => sequence,
        None => continue,
      };

      for entry in sequence.entries() {
        let map = match find_block_map(entry.syntax()) {
          Some(map) => map,
          None => continue,
        };

        if let Some(edit) = sort_map_edit(&map, key_order) {
          edits.push(edit);
        }
      }
    }

    self.apply_edits(edits)
  }

  pub fn validate_each_sort_keys(&self, selector: &str, key_order: &[&str]) -> Result<(), YerbaError> {
    let nodes = if selector.is_empty() {
      match self.navigate(selector) {
        Ok(node) => vec![node],
        Err(_) => return Ok(()),
      }
    } else {
      let found = self.navigate_all_compact(selector);
      if found.is_empty() {
        return Ok(());
      }
      found
    };

    let mut all_unknown: Vec<String> = Vec::new();

    for current_node in &nodes {
      let sequence = match find_block_sequence(current_node) {
        Some(sequence) => sequence,
        None => continue,
      };

      for entry in sequence.entries() {
        if let Some(map) = find_block_map(entry.syntax()) {
          for map_entry in map.entries() {
            if let Some(key_name) = map_entry.key().and_then(|key_node| extract_scalar_text(key_node.syntax())) {
              if !key_order.contains(&key_name.as_str()) && !all_unknown.contains(&key_name) {
                all_unknown.push(key_name);
              }
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

  pub fn sort_items(&mut self, selector: &str, sort_fields: &[SortField], case_sensitive: bool) -> Result<(), YerbaError> {
    self.sort_items_with(selector, sort_fields, case_sensitive, None)
  }

  pub fn sort_items_with(&mut self, selector: &str, sort_fields: &[SortField], case_sensitive: bool, missing: Option<SortMissing>) -> Result<(), YerbaError> {
    if selector.contains("[].") {
      return self.sort_each_items(selector, sort_fields, case_sensitive, missing);
    }

    let current_node = self.navigate(selector)?;

    let sequence = match find_block_sequence(&current_node) {
      Some(sequence) => sequence,
      None => return Ok(()),
    };

    match sort_sequence_edit(&sequence, sort_fields, case_sensitive, missing) {
      Some(edit) => self.apply_edits(vec![edit]),
      None => Ok(()),
    }
  }

  fn sort_each_items(&mut self, selector: &str, sort_fields: &[SortField], case_sensitive: bool, missing: Option<SortMissing>) -> Result<(), YerbaError> {
    let (parent_selector, child_selector) = if let Some(last_bracket) = selector.rfind("[].") {
      (&selector[..last_bracket + 2], &selector[last_bracket + 3..])
    } else {
      (selector, "")
    };

    let parent_nodes = self.navigate_all_compact(parent_selector);
    let mut edits: Vec<(TextRange, String)> = Vec::new();

    for parent_node in &parent_nodes {
      let child_nodes = if child_selector.is_empty() {
        vec![parent_node.clone()]
      } else {
        navigate_from_node(parent_node, child_selector)
      };

      for child_node in &child_nodes {
        let sequence = match find_block_sequence(child_node) {
          Some(sequence) => sequence,
          None => continue,
        };

        if let Some(edit) = sort_sequence_edit(&sequence, sort_fields, case_sensitive, missing) {
          edits.push(edit);
        }
      }
    }

    self.apply_edits(edits)
  }

  pub fn reorder_items(&mut self, selector: &str, by: &str, desired_order: &[&str]) -> Result<(), YerbaError> {
    let field = by.strip_prefix('.').unwrap_or(by);
    let is_scalar = field.is_empty() || field == ".";

    let items_selector = if is_scalar {
      if selector.is_empty() {
        "[]".to_string()
      } else {
        format!("{}[]", selector)
      }
    } else if selector.is_empty() {
      format!("[].{}", field)
    } else {
      format!("{}[].{}", selector, field)
    };

    let labels: Vec<String> = match self.get_value(&items_selector) {
      Some(yaml_serde::Value::Sequence(sequence)) => sequence
        .iter()
        .map(|value| match value {
          yaml_serde::Value::String(string) => string.clone(),
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
        self.move_item(selector, source, target)?;

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

fn sort_map_edit(map: &BlockMap, key_order: &[&str]) -> Option<(TextRange, String)> {
  let entries: Vec<_> = map.entries().collect();

  if entries.len() <= 1 {
    return None;
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
  let original_refs: Vec<&str> = original_keys.iter().map(|key| key.as_str()).collect();

  if sorted_keys == original_refs {
    return None;
  }

  let indent = entries.get(1).map(|entry| preceding_whitespace_indent(entry.syntax())).unwrap_or_default();
  let sorted_groups: Vec<EntryGroup> = keyed.into_iter().map(|(_, group)| group).collect();

  Some((range, rebuild_from_groups(&sorted_groups, &indent, false)))
}

fn sort_sequence_edit(sequence: &BlockSeq, sort_fields: &[SortField], case_sensitive: bool, missing: Option<SortMissing>) -> Option<(TextRange, String)> {
  let entries: Vec<_> = sequence.entries().collect();

  if entries.len() <= 1 {
    return None;
  }

  let (groups, range) = collect_groups_with_range(sequence.syntax());

  let mut sortable: Vec<(Vec<String>, EntryGroup)> = entries
    .iter()
    .zip(groups)
    .map(|(entry, group)| (entry_sort_values(entry, sort_fields), group))
    .collect();

  let original_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

  let compare = |(values_a, _): &(Vec<String>, EntryGroup), (values_b, _): &(Vec<String>, EntryGroup)| {
    compare_sort_values(values_a, values_b, sort_fields, case_sensitive, missing)
  };

  let is_absent = |(values, _): &(Vec<String>, EntryGroup)| values.iter().any(|value| value.is_empty());

  match missing {
    Some(SortMissing::Skip) if sortable.iter().any(is_absent) => return None,
    Some(SortMissing::Keep) => {
      let absent_positions: Vec<bool> = sortable.iter().map(is_absent).collect();
      let (absent, mut present): (Vec<_>, Vec<_>) = sortable.into_iter().partition(is_absent);

      present.sort_by(compare);

      let mut present = present.into_iter();
      let mut absent = absent.into_iter();

      sortable = absent_positions
        .iter()
        .filter_map(|is_absent| if *is_absent { absent.next() } else { present.next() })
        .collect();
    }
    _ => sortable.sort_by(compare),
  }

  let sorted_bodies: Vec<String> = sortable.iter().map(|(_, group)| group.body.clone()).collect();

  if sorted_bodies == original_bodies {
    return None;
  }

  let indent = entries.get(1).map(|entry| preceding_whitespace_indent(entry.syntax())).unwrap_or_default();
  let sorted_groups: Vec<EntryGroup> = sortable.into_iter().map(|(_, group)| group).collect();

  Some((range, rebuild_from_groups(&sorted_groups, &indent, true)))
}

fn entry_sort_values(entry: &yaml_parser::ast::BlockSeqEntry, sort_fields: &[SortField]) -> Vec<String> {
  let scalar_value = || entry.flow().and_then(|flow| extract_scalar_text(flow.syntax())).unwrap_or_default();

  if sort_fields.is_empty() {
    return vec![scalar_value()];
  }

  sort_fields
    .iter()
    .map(|field| {
      if field.selector.is_empty() {
        scalar_value()
      } else {
        let nodes = navigate_from_node(entry.syntax(), &field.selector);
        nodes.first().and_then(extract_scalar_text).unwrap_or_default()
      }
    })
    .collect()
}

fn compare_sort_values(
  values_a: &[String],
  values_b: &[String],
  sort_fields: &[SortField],
  case_sensitive: bool,
  missing: Option<SortMissing>,
) -> std::cmp::Ordering {
  let compare = |value_a: &String, value_b: &String| {
    if case_sensitive {
      value_a.cmp(value_b)
    } else {
      value_a.to_lowercase().cmp(&value_b.to_lowercase())
    }
  };

  let missing_ordering = |value_a: &String, value_b: &String| match (missing, value_a.is_empty(), value_b.is_empty()) {
    (Some(SortMissing::First), true, false) => Some(std::cmp::Ordering::Less),
    (Some(SortMissing::First), false, true) => Some(std::cmp::Ordering::Greater),
    (Some(SortMissing::Last), true, false) => Some(std::cmp::Ordering::Greater),
    (Some(SortMissing::Last), false, true) => Some(std::cmp::Ordering::Less),
    _ => None,
  };

  for (index, field) in sort_fields.iter().enumerate().take(values_a.len()) {
    if let Some(ordering) = missing_ordering(&values_a[index], &values_b[index]) {
      return ordering;
    }

    let ordering = compare(&values_a[index], &values_b[index]);
    let ordering = if field.ascending { ordering } else { ordering.reverse() };

    if ordering != std::cmp::Ordering::Equal {
      return ordering;
    }
  }

  if sort_fields.is_empty() && !values_a.is_empty() && !values_b.is_empty() {
    return missing_ordering(&values_a[0], &values_b[0]).unwrap_or_else(|| compare(&values_a[0], &values_b[0]));
  }

  std::cmp::Ordering::Equal
}

fn strip_bracket_suffix(selector: &str) -> Option<&str> {
  if selector == "[]" {
    Some("")
  } else if let Some(stripped) = selector.strip_suffix(".[]") {
    Some(stripped)
  } else {
    selector.strip_suffix("[]")
  }
}
