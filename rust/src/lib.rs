pub mod didyoumean;
mod document;
pub mod error;
pub mod ffi;
pub mod json;
mod quote_style;
pub mod schema;
pub mod selector;
mod syntax;
mod yaml_writer;
pub mod yerbafile;

pub use document::style::StyleEnforcement;
pub use document::{
  collect_selectors, validate_condition, validate_item_condition, Document, DuplicateInfo, InsertPosition, LocatedNode, Location, NodeInfo, NodeType, SortField,
};
pub use error::YerbaError;
pub use quote_style::{KeyStyle, QuoteStyle};
pub use selector::Selector;
pub use syntax::{detect_yaml_type, ScalarValue, YerbaValueType};
pub use yaml_writer::json_to_yaml_text;
pub use yerbafile::Yerbafile;

pub fn version() -> &'static str {
  env!("CARGO_PKG_VERSION")
}

pub fn parse(source: &str) -> Result<Document, YerbaError> {
  Document::parse(source)
}

pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<Document, YerbaError> {
  Document::parse_file(path)
}

pub fn glob_get(pattern: &str, selector: &str) -> Vec<document::LocatedNode> {
  use rayon::prelude::*;

  let files = match glob::glob(pattern) {
    Ok(paths) => paths.filter_map(|p| p.ok()).collect::<Vec<_>>(),
    Err(_) => return vec![],
  };

  files
    .par_iter()
    .flat_map(|file| {
      let mut results = Vec::new();

      if let Ok(document) = Document::parse_file(file) {
        results.extend(document.get_all_located(selector));
      }

      results
    })
    .collect()
}

pub fn glob_find(pattern: &str, selector: &str, condition: Option<&str>, select: Option<&str>) -> Result<Vec<serde_json::Value>, YerbaError> {
  use rayon::prelude::*;

  if let Some(cond) = condition {
    crate::document::validate_condition(cond)?;
  }

  let files = match glob::glob(pattern) {
    Ok(paths) => paths.filter_map(|p| p.ok()).collect::<Vec<_>>(),
    Err(_) => return Ok(vec![]),
  };

  let select_fields: Option<Vec<&str>> = select.map(|s| s.split(',').collect());

  let results: Vec<serde_json::Value> = files
    .par_iter()
    .flat_map(|file| {
      let mut file_results = Vec::new();

      if let Ok(document) = Document::parse_file(file) {
        let values = match condition {
          Some(cond) => document.filter(selector, cond).unwrap_or_default(),
          None => document.get_values(selector),
        };

        let file_string = file.to_string_lossy().to_string();

        for value in &values {
          let mut result = serde_json::Map::new();
          result.insert("__file".to_string(), serde_json::Value::String(file_string.clone()));

          match &select_fields {
            Some(fields) => {
              for field in fields {
                let json_value = json::resolve_select_field(value, field);
                let json_key = json::select_field_key(field);
                result.insert(json_key, json_value);
              }
            }
            None => {
              if let yaml_serde::Value::Mapping(map) = value {
                for (key, yaml_value) in map {
                  let json_key = match key {
                    yaml_serde::Value::String(string) => string.clone(),
                    _ => format!("{:?}", key),
                  };

                  result.insert(json_key, json::yaml_to_json(yaml_value));
                }
              }
            }
          }

          file_results.push(serde_json::Value::Object(result));
        }
      }

      file_results
    })
    .collect();

  Ok(results)
}
