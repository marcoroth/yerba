use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Document, QuoteStyle, YerbaError};

#[derive(Debug, Deserialize)]
pub struct Yerbafile {
  #[serde(default)]
  pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
  pub files: String,
  #[serde(default)]
  pub path: Option<String>,
  pub pipeline: Vec<PipelineStep>,
}

#[derive(Debug, Clone)]
pub enum PipelineStep {
  Get(GetConfig),
  SortKeys(SortKeysConfig),
  QuoteStyle(QuoteStyleConfig),
  Set(SetConfig),
  Insert(InsertConfig),
  Delete(DeleteConfig),
  Rename(RenameConfig),
  Remove(RemoveConfig),
  BlankLines(BlankLinesConfig),
  Sort(SortConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortConfig {
  #[serde(default)]
  pub path: Option<String>,
  #[serde(default)]
  pub by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlankLinesConfig {
  #[serde(default)]
  pub path: Option<String>,
  pub count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetConfig {
  pub path: String,
  #[serde(rename = "as")]
  pub as_name: String,
  #[serde(default)]
  pub file: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Variable {
  Single(String),
  List(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RenameConfig {
  pub from: String,
  pub to: String,
  #[serde(default)]
  pub condition: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RemoveConfig {
  pub path: String,
  pub value: String,
  #[serde(default)]
  pub condition: Option<String>,
}

impl<'de> Deserialize<'de> for PipelineStep {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let mapping = serde_yaml::Mapping::deserialize(deserializer)?;

    if let Some(value) = mapping.get(serde_yaml::Value::String("get".to_string())) {
      let config: GetConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Get(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("sort_keys".to_string())) {
      let config: SortKeysConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::SortKeys(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("quote_style".to_string())) {
      let config: QuoteStyleConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::QuoteStyle(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("set".to_string())) {
      let config: SetConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Set(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("insert".to_string())) {
      let config: InsertConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Insert(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("delete".to_string())) {
      let config: DeleteConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Delete(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("rename".to_string())) {
      let config: RenameConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Rename(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("remove".to_string())) {
      let config: RemoveConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Remove(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("blank_lines".to_string())) {
      let config: BlankLinesConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::BlankLines(config));
    }

    if let Some(value) = mapping.get(serde_yaml::Value::String("sort".to_string())) {
      let config: SortConfig = serde_yaml::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Sort(config));
    }

    Err(serde::de::Error::custom(
      "unknown pipeline step: expected get, sort_keys, quote_style, set, insert, delete, rename, remove, blank_lines, or sort",
    ))
  }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortKeysConfig {
  #[serde(default)]
  pub path: Option<String>,
  pub order: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuoteStyleConfig {
  #[serde(default = "default_key_style")]
  pub key_style: String,
  pub value_style: String,
  #[serde(default)]
  pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetConfig {
  pub path: String,
  pub value: String,
  #[serde(default)]
  pub condition: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InsertConfig {
  pub path: String,
  pub value: String,
  #[serde(default)]
  pub condition: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteConfig {
  pub path: String,
  #[serde(default)]
  pub condition: Option<String>,
}

fn default_key_style() -> String {
  "plain".to_string()
}

#[derive(Debug)]
pub struct RuleResult {
  pub file: String,
  pub changed: bool,
  pub error: Option<String>,
}

impl Yerbafile {
  pub fn load(path: impl AsRef<Path>) -> Result<Self, YerbaError> {
    let content = fs::read_to_string(path.as_ref())?;
    let yerbafile: Yerbafile =
      serde_yaml::from_str(&content).map_err(|error| YerbaError::ParseError(format!("{}", error)))?;
    Ok(yerbafile)
  }

  pub fn find() -> Option<PathBuf> {
    let candidates = ["Yerbafile", "Yerbafile.yml", "Yerbafile.yaml", ".yerbafile"];

    let mut directory = std::env::current_dir().ok()?;

    loop {
      for candidate in &candidates {
        let path = directory.join(candidate);

        if path.exists() {
          return Some(path);
        }
      }

      if !directory.pop() {
        return None;
      }
    }
  }

  pub fn sort_order_for(&self, file_path: &str, dot_path: &str) -> Option<Vec<String>> {
    for rule in &self.rules {
      let sort_keys_config = rule.pipeline.iter().find_map(|step| match step {
        PipelineStep::SortKeys(config) => Some(config),
        _ => None,
      });

      if let Some(config) = sort_keys_config {
        let full_path = resolve_step_path(rule.path.as_deref(), config.path.as_deref());
        let normalized = full_path.trim_end_matches("[]").trim_end_matches('.');

        if normalized != dot_path && !normalized.is_empty() && !dot_path.is_empty() {
          continue;
        }

        if let Ok(pattern) = glob::Pattern::new(&rule.files) {
          if pattern.matches(file_path) || pattern.matches_path(Path::new(file_path)) {
            return Some(config.order.clone());
          }
        }
      }
    }

    None
  }

  pub fn apply(&self, write: bool) -> Vec<RuleResult> {
    let mut results = Vec::new();

    for rule in &self.rules {
      let files = match glob::glob(&rule.files) {
        Ok(paths) => paths.filter_map(|entry| entry.ok()).collect::<Vec<_>>(),

        Err(error) => {
          results.push(RuleResult {
            file: rule.files.clone(),
            changed: false,
            error: Some(format!("invalid glob: {}", error)),
          });

          continue;
        }
      };

      let file_strings: Vec<String> = files.iter().map(|path| path.to_string_lossy().to_string()).collect();

      let mut has_validation_error = false;

      for step in &rule.pipeline {
        if let PipelineStep::SortKeys(config) = step {
          let full_path = resolve_step_path(rule.path.as_deref(), config.path.as_deref());
          let key_order: Vec<&str> = config.order.iter().map(|key| key.as_str()).collect();

          let validation_results: Vec<RuleResult> = file_strings
            .par_iter()
            .filter_map(|file| {
              let document = match Document::parse_file(file) {
                Ok(document) => document,
                Err(error) => {
                  return Some(RuleResult {
                    file: file.clone(),
                    changed: false,
                    error: Some(format!("{}", error)),
                  });
                }
              };

              if let Err(error) = document.validate_sort_keys(&full_path, &key_order) {
                Some(RuleResult {
                  file: file.clone(),
                  changed: false,
                  error: Some(format!("{}", error)),
                })
              } else {
                None
              }
            })
            .collect();

          if !validation_results.is_empty() {
            has_validation_error = true;
            results.extend(validation_results);
          }
        }
      }

      if has_validation_error {
        continue;
      }

      let file_results: Vec<RuleResult> = file_strings
        .par_iter()
        .map(|file| self.apply_pipeline_to_file(rule, file, write))
        .collect();

      results.extend(file_results);
    }

    results
  }

  fn apply_pipeline_to_file(&self, rule: &Rule, file: &str, write: bool) -> RuleResult {
    let mut document = match Document::parse_file(file) {
      Ok(document) => document,
      Err(error) => {
        return RuleResult {
          file: file.to_string(),
          changed: false,
          error: Some(format!("{}", error)),
        }
      }
    };

    let original = document.to_string();
    let base_path = rule.path.as_deref();
    let mut variables: HashMap<String, Variable> = HashMap::new();

    for step in &rule.pipeline {
      if let Err(error) = execute_step(&mut document, step, base_path, &mut variables) {
        return RuleResult {
          file: file.to_string(),
          changed: false,
          error: Some(format!("{}", error)),
        };
      }
    }

    let new_content = document.to_string();
    let changed = new_content != original;

    if changed && write {
      if let Err(error) = fs::write(file, &new_content) {
        return RuleResult {
          file: file.to_string(),
          changed,
          error: Some(format!("{}", error)),
        };
      }
    }

    RuleResult {
      file: file.to_string(),
      changed,
      error: None,
    }
  }
}

fn execute_step(
  document: &mut Document,
  step: &PipelineStep,
  base_path: Option<&str>,
  variables: &mut HashMap<String, Variable>,
) -> Result<(), YerbaError> {
  match step {
    PipelineStep::Get(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));

      if let Some(file_pattern) = &config.file {
        let mut all_values = Vec::new();

        let files =
          glob::glob(file_pattern).map_err(|error| YerbaError::ParseError(format!("invalid glob: {}", error)))?;

        for entry in files.flatten() {
          let external_document = Document::parse_file(&entry)?;
          all_values.extend(external_document.get_all(&config.path));
        }

        if all_values.len() == 1 && !config.path.contains('[') {
          variables.insert(config.as_name.clone(), Variable::Single(all_values.remove(0)));
        } else {
          variables.insert(config.as_name.clone(), Variable::List(all_values));
        }
      } else if config.path.contains('[') {
        let values = document.get_all(&full_path);

        variables.insert(config.as_name.clone(), Variable::List(values));
      } else {
        let value = document
          .get(&full_path)
          .ok_or_else(|| YerbaError::PathNotFound(full_path.clone()))?;
        variables.insert(config.as_name.clone(), Variable::Single(value));
      }

      Ok(())
    }

    PipelineStep::QuoteStyle(config) => {
      let dot_path = config.path.as_deref();

      let key_style = config.key_style.parse::<QuoteStyle>().map_err(YerbaError::ParseError)?;

      let value_style = config
        .value_style
        .parse::<QuoteStyle>()
        .map_err(YerbaError::ParseError)?;

      document.enforce_key_style(&key_style, dot_path)?;
      document.enforce_quotes_at(&value_style, dot_path)?;

      Ok(())
    }

    PipelineStep::SortKeys(config) => {
      let full_path = resolve_step_path(base_path, config.path.as_deref());
      let key_order: Vec<&str> = config.order.iter().map(|key| key.as_str()).collect();

      document.sort_keys(&full_path, &key_order)
    }

    PipelineStep::Set(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));
      let resolved_value = resolve_template(&config.value, document, base_path, variables)?;

      if let Some(condition) = &config.condition {
        let resolved_condition = resolve_template(condition, document, base_path, variables)?;
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, &resolved_condition) {
          return Ok(());
        }
      }

      document.set(&full_path, &resolved_value)
    }

    PipelineStep::Insert(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));
      let resolved_value = resolve_template(&config.value, document, base_path, variables)?;

      if let Some(condition) = &config.condition {
        let resolved_condition = resolve_template(condition, document, base_path, variables)?;
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, &resolved_condition) {
          return Ok(());
        }
      }

      document.insert_into(&full_path, &resolved_value, crate::InsertPosition::Last)
    }

    PipelineStep::Delete(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));

      if let Some(condition) = &config.condition {
        let resolved_condition = resolve_template(condition, document, base_path, variables)?;
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, &resolved_condition) {
          return Ok(());
        }
      }

      document.delete(&full_path)
    }

    PipelineStep::Rename(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.from));

      if let Some(condition) = &config.condition {
        let resolved_condition = resolve_template(condition, document, base_path, variables)?;
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, &resolved_condition) {
          return Ok(());
        }
      }

      document.rename(&full_path, &config.to)
    }

    PipelineStep::Remove(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));
      let resolved_value = resolve_template(&config.value, document, base_path, variables)?;

      if let Some(condition) = &config.condition {
        let resolved_condition = resolve_template(condition, document, base_path, variables)?;
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, &resolved_condition) {
          return Ok(());
        }
      }

      document.remove(&full_path, &resolved_value)
    }

    PipelineStep::BlankLines(config) => {
      let full_path = resolve_step_path(base_path, config.path.as_deref());

      document.enforce_blank_lines(&full_path, config.count)
    }

    PipelineStep::Sort(config) => {
      let full_path = resolve_step_path(base_path, config.path.as_deref());
      let sort_fields = config.by.as_deref().map(crate::SortField::parse_list).unwrap_or_default();

      document.sort_items(&full_path, &sort_fields)
    }
  }
}

pub fn resolve_template(
  template: &str,
  document: &Document,
  base_path: Option<&str>,
  variables: &HashMap<String, Variable>,
) -> Result<String, YerbaError> {
  if !template.contains("${") {
    return Ok(template.to_string());
  }

  let mut result = String::new();
  let mut rest = template;

  while let Some(start) = rest.find("${") {
    result.push_str(&rest[..start]);

    let after_dollar = &rest[start + 2..];

    let end = after_dollar
      .find('}')
      .ok_or_else(|| YerbaError::ParseError("unclosed ${ in template".to_string()))?;

    let reference = &after_dollar[..end];

    let resolved = if let Some(variable) = variables.get(reference) {
      match variable {
        Variable::Single(value) => value.clone(),
        Variable::List(values) => values.join(", "),
      }
    } else {
      let full_path = resolve_step_path(base_path, Some(reference));

      document
        .get(&full_path)
        .ok_or_else(|| YerbaError::ReferenceNotFound(reference.to_string()))?
    };

    result.push_str(&resolved);
    rest = &after_dollar[end + 1..];
  }

  result.push_str(rest);

  Ok(result)
}

fn resolve_step_path(base_path: Option<&str>, step_path: Option<&str>) -> String {
  let base = base_path.unwrap_or("");
  let step = step_path.unwrap_or("");

  match (base.is_empty(), step.is_empty()) {
    (true, true) => String::new(),
    (true, false) => step.to_string(),
    (false, true) => base.to_string(),
    (false, false) => format!("{}.{}", base, step),
  }
}
