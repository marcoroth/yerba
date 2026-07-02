use super::*;

impl Document {
  pub fn validate_schema(&self, schema: &serde_json::Value, items: bool, selector: Option<&str>) -> Vec<crate::schema::ValidationError> {
    let path = selector.unwrap_or("");
    let has_wildcard = crate::selector::Selector::parse(path).has_wildcard();

    let value = match self.get_value(path) {
      Some(value) => crate::json::yaml_to_json(&value),
      None => return vec![],
    };

    let validate_items = items || has_wildcard;

    let mut errors = if validate_items {
      match value.as_array() {
        Some(array) => crate::schema::validate_array(array, schema),
        None if value.is_null() => vec![crate::schema::ValidationError {
          path: String::new(),
          message: "expected an array but document is empty or not a sequence".to_string(),
          item_label: None,
          line: None,
        }],
        None => crate::schema::validate_array(std::slice::from_ref(&value), schema),
      }
    } else {
      if value.is_null() {
        return vec![];
      }

      crate::schema::validate_value(&value, schema)
    };

    let source = self.source_text();

    for error in &mut errors {
      if !error.path.is_empty() {
        let selector = json_pointer_to_selector(&error.path);

        if let Ok(node) = self.navigate(&selector) {
          let offset: usize = node.text_range().start().into();
          let line = source[..offset].chars().filter(|c| *c == '\n').count() + 1;

          error.line = Some(line);
        }
      }
    }

    errors
  }
}

fn json_pointer_to_selector(pointer: &str) -> String {
  pointer
    .strip_prefix('/')
    .unwrap_or(pointer)
    .split('/')
    .map(|segment| {
      if let Ok(index) = segment.parse::<usize>() {
        format!("[{}]", index)
      } else {
        segment.to_string()
      }
    })
    .fold(String::new(), |mut path, segment| {
      if !path.is_empty() && !segment.starts_with('[') {
        path.push('.');
      }

      path.push_str(&segment);
      path
    })
}
