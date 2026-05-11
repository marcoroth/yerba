use std::fs;
use std::path::Path;

use crate::error::YerbaError;

#[derive(Debug)]
pub struct ValidationError {
  pub path: String,
  pub message: String,
  pub item_label: Option<String>,
  pub line: Option<usize>,
}

impl std::fmt::Display for ValidationError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let location = match self.line {
      Some(line) => format!(" (line {})", line),
      None => String::new(),
    };

    match &self.item_label {
      Some(label) => write!(f, "{} at {} ({}){}", self.message, self.path, label, location),
      None => write!(f, "{} at {}{}", self.message, self.path, location),
    }
  }
}

pub fn validate_value(value: &serde_json::Value, schema: &serde_json::Value) -> Vec<ValidationError> {
  let compiled = match jsonschema::draft7::new(schema) {
    Ok(compiled) => compiled,
    Err(error) => {
      return vec![ValidationError {
        path: String::new(),
        message: format!("invalid schema: {}", error),
        item_label: None,
        line: None,
      }];
    }
  };

  compiled
    .iter_errors(value)
    .map(|error| ValidationError {
      path: error.instance_path.to_string(),
      message: error.to_string(),
      item_label: None,
      line: None,
    })
    .collect()
}

pub fn validate_array(items: &[serde_json::Value], schema: &serde_json::Value) -> Vec<ValidationError> {
  let compiled = match jsonschema::draft7::new(schema) {
    Ok(compiled) => compiled,
    Err(error) => {
      return vec![ValidationError {
        path: String::new(),
        message: format!("invalid schema: {}", error),
        item_label: None,
        line: None,
      }];
    }
  };

  let mut errors = Vec::new();

  for (index, item) in items.iter().enumerate() {
    let item_label = item
      .get("name")
      .or_else(|| item.get("title"))
      .or_else(|| item.get("id"))
      .and_then(|value| value.as_str())
      .map(|string| string.to_string())
      .unwrap_or_else(|| format!("index {}", index));

    for error in compiled.iter_errors(item) {
      errors.push(ValidationError {
        path: format!("/{}{}", index, error.instance_path),
        message: error.to_string(),
        item_label: Some(item_label.clone()),
        line: None,
      });
    }
  }

  errors
}

pub fn load_schema(path: impl AsRef<Path>) -> Result<serde_json::Value, YerbaError> {
  let content = fs::read_to_string(path.as_ref())?;

  serde_json::from_str(&content).map_err(|error| YerbaError::ParseError(format!("invalid JSON schema: {}", error)))
}
