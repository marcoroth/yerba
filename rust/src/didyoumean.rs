pub fn didyoumean(input: &str, list: &[String]) -> Option<String> {
  if list.is_empty() {
    return None;
  }

  let input_lower = input.to_lowercase();

  let mut scored: Vec<(String, usize)> = list
    .iter()
    .map(|item| (item.clone(), levenshtein(&input_lower, &item.to_lowercase())))
    .collect();

  scored.sort_by(|(a_item, a_distance), (b_item, b_distance)| a_distance.cmp(b_distance).then_with(|| a_item.cmp(b_item)));

  scored.into_iter().next().map(|(item, _)| item)
}

pub fn didyoumean_ranked(input: &str, list: &[String], threshold: usize) -> Vec<String> {
  if list.is_empty() {
    return Vec::new();
  }

  let input_lower = input.to_lowercase();

  let mut scored: Vec<(String, usize)> = list
    .iter()
    .map(|item| (item.clone(), levenshtein(&input_lower, &item.to_lowercase())))
    .filter(|(_, distance)| *distance <= threshold)
    .collect();

  scored.sort_by(|(a_item, a_distance), (b_item, b_distance)| a_distance.cmp(b_distance).then_with(|| a_item.cmp(b_item)));

  scored.into_iter().map(|(item, _)| item).collect()
}

fn levenshtein(a: &str, b: &str) -> usize {
  let b_length = b.len();
  let mut previous: Vec<usize> = (0..=b_length).collect();
  let mut current = vec![0; b_length + 1];

  for (i, a_character) in a.chars().enumerate() {
    current[0] = i + 1;

    for (j, b_character) in b.chars().enumerate() {
      let cost = if a_character == b_character { 0 } else { 1 };
      current[j + 1] = (previous[j] + cost).min(previous[j + 1] + 1).min(current[j] + 1);
    }

    std::mem::swap(&mut previous, &mut current);
  }

  previous[b_length]
}
