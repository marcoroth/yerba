use rayon::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Document, QuoteStyle, YerbaError};

#[derive(Debug, Deserialize)]
pub struct Yerbafile {
  #[serde(default)]
  pub files: Option<String>,
  #[serde(default)]
  pub pipeline: Vec<PipelineStep>,
  #[serde(default)]
  pub rules: Vec<Rule>,
  #[serde(skip)]
  pub directory: Option<PathBuf>,
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
  SortKeys(SortKeysConfig),
  QuoteStyle(QuoteStyleConfig),
  CollectionStyle(CollectionStyleConfig),
  SequenceIndent(SequenceIndentConfig),
  Set(SetConfig),
  Insert(InsertConfig),
  Delete(DeleteConfig),
  Rename(RenameConfig),
  Remove(RemoveConfig),
  BlankLines(BlankLinesConfig),
  Sort(SortConfig),
  Directives(DirectivesConfig),
  Unique(UniqueConfig),
  Schema(SchemaConfig),
  FinalNewline(FinalNewlineConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortConfig {
  #[serde(default)]
  pub path: Option<String>,
  #[serde(default)]
  pub by: Option<String>,
  #[serde(default)]
  pub case_sensitive: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlankLinesConfig {
  #[serde(default)]
  pub path: Option<String>,
  pub count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectivesConfig {
  #[serde(default)]
  pub ensure: bool,
  #[serde(default)]
  pub remove: bool,
  #[serde(default)]
  pub max: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniqueConfig {
  #[serde(default)]
  pub path: Option<String>,
  #[serde(default = "default_dot")]
  pub by: String,
  #[serde(default)]
  pub remove: bool,
  #[serde(default)]
  pub allow_blank_duplicates: bool,
}

fn default_dot() -> String {
  ".".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchemaConfig {
  pub file: String,
  #[serde(default)]
  pub path: Option<String>,
  #[serde(default)]
  pub items: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FinalNewlineConfig {
  #[serde(default = "default_one")]
  pub count: usize,
}

fn default_one() -> usize {
  1
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
    let mapping = yaml_serde::Mapping::deserialize(deserializer)?;

    if let Some(value) = mapping.get(yaml_serde::Value::String("sort_keys".to_string())) {
      let config: SortKeysConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::SortKeys(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("quote_style".to_string())) {
      let config: QuoteStyleConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::QuoteStyle(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("collection_style".to_string())) {
      let config: CollectionStyleConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::CollectionStyle(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("sequence_indent".to_string())) {
      let config: SequenceIndentConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::SequenceIndent(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("set".to_string())) {
      let config: SetConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Set(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("insert".to_string())) {
      let config: InsertConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Insert(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("delete".to_string())) {
      let config: DeleteConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Delete(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("rename".to_string())) {
      let config: RenameConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Rename(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("remove".to_string())) {
      let config: RemoveConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Remove(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("blank_lines".to_string())) {
      let config: BlankLinesConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::BlankLines(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("directives".to_string())) {
      let config: DirectivesConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Directives(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("sort".to_string())) {
      let config: SortConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Sort(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("unique".to_string())) {
      let config: UniqueConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Unique(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("schema".to_string())) {
      let config: SchemaConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::Schema(config));
    }

    if let Some(value) = mapping.get(yaml_serde::Value::String("final_newline".to_string())) {
      let config: FinalNewlineConfig = yaml_serde::from_value(value.clone()).map_err(serde::de::Error::custom)?;
      return Ok(PipelineStep::FinalNewline(config));
    }

    Err(serde::de::Error::custom(
      "unknown pipeline step: expected sort_keys, quote_style, collection_style, sequence_indent, set, insert, delete, rename, remove, blank_lines, sort, directives, unique, schema, or final_newline",
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
pub struct CollectionStyleConfig {
  pub style: String,
  #[serde(default)]
  pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SequenceIndentConfig {
  pub style: String,
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
  pub error: Option<YerbaError>,
}

impl Yerbafile {
  pub fn load(path: impl AsRef<Path>) -> Result<Self, YerbaError> {
    let content = fs::read_to_string(path.as_ref())?;
    let mut yerbafile: Yerbafile = yaml_serde::from_str(&content).map_err(|error| YerbaError::ParseError(format!("{}", error)))?;
    yerbafile.directory = path.as_ref().parent().map(|p| p.to_path_buf());
    yerbafile.validate_conditions()?;
    Ok(yerbafile)
  }

  fn validate_conditions(&self) -> Result<(), YerbaError> {
    let pipelines = std::iter::once(&self.pipeline).chain(self.rules.iter().map(|rule| &rule.pipeline));

    for pipeline in pipelines {
      for step in pipeline {
        let condition = match step {
          PipelineStep::Set(config) => config.condition.as_deref(),
          PipelineStep::Insert(config) => config.condition.as_deref(),
          PipelineStep::Delete(config) => config.condition.as_deref(),
          PipelineStep::Rename(config) => config.condition.as_deref(),
          PipelineStep::Remove(config) => config.condition.as_deref(),
          _ => None,
        };

        if let Some(condition) = condition {
          crate::validate_condition(condition)?;
        }
      }
    }

    Ok(())
  }

  pub fn resolve_path(&self, relative: &str) -> PathBuf {
    match &self.directory {
      Some(directory) => directory.join(relative),
      None => PathBuf::from(relative),
    }
  }

  pub fn find() -> Option<PathBuf> {
    Self::find_from(std::env::current_dir().ok()?)
  }

  pub fn find_from(start: impl AsRef<Path>) -> Option<PathBuf> {
    let candidates = ["Yerbafile", "Yerbafile.yml", "Yerbafile.yaml", ".yerbafile"];

    let mut directory = start.as_ref().to_path_buf();

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
    let mut globally_processed: std::collections::HashSet<String> = std::collections::HashSet::new();

    for rule in &self.rules {
      let files = match glob::glob(&rule.files) {
        Ok(paths) => paths.filter_map(|entry| entry.ok()).collect::<Vec<_>>(),

        Err(error) => {
          results.push(RuleResult {
            file: rule.files.clone(),
            changed: false,
            error: Some(YerbaError::ParseError(format!("invalid glob: {}", error))),
          });

          continue;
        }
      };

      let file_strings: Vec<String> = files.iter().map(|path| path.to_string_lossy().to_string()).collect();

      let file_results: Vec<RuleResult> = file_strings
        .par_iter()
        .map(|file| {
          let run_global = !globally_processed.contains(file.as_str()) && self.should_run_global_pipeline(file);
          self.apply_pipeline_to_file(rule, file, write, run_global)
        })
        .collect();

      for result in &file_results {
        globally_processed.insert(result.file.clone());
      }

      results.extend(file_results);
    }

    if !self.pipeline.is_empty() {
      if let Some(ref global_glob) = self.files {
        if let Ok(paths) = glob::glob(global_glob) {
          let unprocessed: Vec<String> = paths
            .filter_map(|entry| entry.ok())
            .map(|path| path.to_string_lossy().to_string())
            .filter(|file| !globally_processed.contains(file.as_str()))
            .collect();

          let global_results: Vec<RuleResult> = unprocessed.par_iter().map(|file| self.apply_global_pipeline_to_file(file, write)).collect();

          results.extend(global_results);
        }
      }
    }

    results
  }

  fn apply_pipeline_to_file(&self, rule: &Rule, file: &str, write: bool, run_global: bool) -> RuleResult {
    let mut document = match Document::parse_file(file) {
      Ok(document) => document,
      Err(error) => {
        return RuleResult {
          file: file.to_string(),
          changed: false,
          error: Some(error),
        }
      }
    };

    let original = document.to_string();

    if run_global {
      if let Err(error) = execute_pipeline(&mut document, &self.pipeline, None, file, self) {
        return RuleResult {
          file: file.to_string(),
          changed: false,
          error: Some(error),
        };
      }
    }

    let base_path = rule.path.as_deref();

    for step in &rule.pipeline {
      if let Err(error) = execute_step(&mut document, step, base_path, file, self) {
        return RuleResult {
          file: file.to_string(),
          changed: false,
          error: Some(error),
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
          error: Some(YerbaError::IoError(error)),
        };
      }
    }

    RuleResult {
      file: file.to_string(),
      changed,
      error: None,
    }
  }

  pub fn apply_file(&self, file: &str, write: bool) -> Vec<RuleResult> {
    let mut results = Vec::new();
    let mut ran_global = false;

    for rule in &self.rules {
      if let Ok(pattern) = glob::Pattern::new(&rule.files) {
        if !pattern.matches(file) && !pattern.matches_path(Path::new(file)) {
          continue;
        }
      } else {
        continue;
      }

      let run_global = !ran_global && self.should_run_global_pipeline(file);
      results.push(self.apply_pipeline_to_file(rule, file, write, run_global));
      ran_global = true;
    }

    if !ran_global && self.should_run_global_pipeline(file) {
      results.push(self.apply_global_pipeline_to_file(file, write));
    }

    results
  }

  fn apply_global_pipeline_to_file(&self, file: &str, write: bool) -> RuleResult {
    let mut document = match Document::parse_file(file) {
      Ok(document) => document,
      Err(error) => {
        return RuleResult {
          file: file.to_string(),
          changed: false,
          error: Some(error),
        }
      }
    };

    let original = document.to_string();

    if let Err(error) = execute_pipeline(&mut document, &self.pipeline, None, file, self) {
      return RuleResult {
        file: file.to_string(),
        changed: false,
        error: Some(error),
      };
    }

    let new_content = document.to_string();
    let changed = new_content != original;

    if changed && write {
      if let Err(error) = fs::write(file, &new_content) {
        return RuleResult {
          file: file.to_string(),
          changed,
          error: Some(YerbaError::IoError(error)),
        };
      }
    }

    RuleResult {
      file: file.to_string(),
      changed,
      error: None,
    }
  }

  fn relativize_path(&self, file_path: &str) -> Option<String> {
    let path = Path::new(file_path);

    if !path.is_absolute() {
      return None;
    }

    if let Some(directory) = &self.directory {
      if let Ok(relative) = path.strip_prefix(directory) {
        return Some(relative.to_string_lossy().to_string());
      }
    }

    std::env::current_dir()
      .ok()
      .and_then(|cwd| path.strip_prefix(&cwd).ok().map(|relative| relative.to_string_lossy().to_string()))
  }

  fn should_run_global_pipeline(&self, file_path: &str) -> bool {
    if self.pipeline.is_empty() {
      return false;
    }

    match &self.files {
      Some(glob_pattern) => glob::Pattern::new(glob_pattern)
        .map(|pattern| pattern.matches(file_path) || pattern.matches_path(Path::new(file_path)))
        .unwrap_or(false),
      None => true,
    }
  }

  pub fn apply_to_document(&self, document: &mut Document, file_path: &str) -> Result<bool, YerbaError> {
    let original = document.to_string();
    let relative_path = self.relativize_path(file_path);
    let match_path = relative_path.as_deref().unwrap_or(file_path);

    if self.should_run_global_pipeline(match_path) {
      execute_pipeline(document, &self.pipeline, None, file_path, self)?;
    }

    for rule in &self.rules {
      if !match_path.is_empty() {
        if let Ok(pattern) = glob::Pattern::new(&rule.files) {
          if !pattern.matches(match_path) && !pattern.matches_path(Path::new(match_path)) {
            continue;
          }
        } else {
          continue;
        }
      }

      let base_path = rule.path.as_deref();

      for step in &rule.pipeline {
        execute_step(document, step, base_path, file_path, self)?;
      }
    }

    Ok(document.to_string() != original)
  }
}

fn execute_pipeline(document: &mut Document, steps: &[PipelineStep], base_path: Option<&str>, file: &str, yerbafile: &Yerbafile) -> Result<(), YerbaError> {
  use crate::document::style::StyleEnforcement;

  let mut enforcement = StyleEnforcement {
    collection_style: None,
    sequence_indent: None,
    key_style: None,
    value_style: None,
  };

  let mut has_enforcement = false;
  let mut remaining_steps: Vec<&PipelineStep> = Vec::new();

  for step in steps {
    match step {
      PipelineStep::CollectionStyle(config) if config.path.is_none() && base_path.is_none() => {
        enforcement.collection_style = Some(config.style.clone());
        has_enforcement = true;
      }

      PipelineStep::SequenceIndent(config) if config.path.is_none() && base_path.is_none() => {
        enforcement.sequence_indent = Some(config.style.clone());
        has_enforcement = true;
      }

      PipelineStep::QuoteStyle(config) if config.path.is_none() && base_path.is_none() => {
        let key_style = config.key_style.parse::<crate::KeyStyle>().ok();
        let value_style = config.value_style.parse::<QuoteStyle>().ok();

        enforcement.key_style = key_style;
        enforcement.value_style = value_style;
        has_enforcement = true;
      }

      _ => remaining_steps.push(step),
    }
  }

  if has_enforcement {
    document.enforce_styles(&enforcement)?;
  }

  for step in remaining_steps {
    execute_step(document, step, base_path, file, yerbafile)?;
  }

  Ok(())
}

fn execute_step(document: &mut Document, step: &PipelineStep, base_path: Option<&str>, _file: &str, yerbafile: &Yerbafile) -> Result<(), YerbaError> {
  match step {
    PipelineStep::QuoteStyle(config) => {
      let dot_path = config.path.as_deref();

      let key_style = config.key_style.parse::<crate::KeyStyle>().map_err(YerbaError::ParseError)?;

      let value_style = config.value_style.parse::<QuoteStyle>().map_err(YerbaError::ParseError)?;

      document.enforce_key_style(&key_style, dot_path)?;
      let warnings = document.enforce_quotes_at(&value_style, dot_path)?;

      for warning in &warnings {
        eprintln!("  warning: {}", warning);
      }

      Ok(())
    }

    PipelineStep::CollectionStyle(config) => {
      let dot_path = resolve_step_path(base_path, config.path.as_deref());
      let path = if dot_path.is_empty() { None } else { Some(dot_path.as_str()) };

      document.enforce_collection_style(&config.style, path)
    }

    PipelineStep::SequenceIndent(config) => {
      let dot_path = resolve_step_path(base_path, config.path.as_deref());
      let path = if dot_path.is_empty() { None } else { Some(dot_path.as_str()) };

      document.enforce_sequence_indent(&config.style, path)
    }

    PipelineStep::SortKeys(config) => {
      let full_path = resolve_step_path(base_path, config.path.as_deref());
      let key_order: Vec<&str> = config.order.iter().map(|key| key.as_str()).collect();

      document.validate_sort_keys(&full_path, &key_order)?;
      document.sort_keys(&full_path, &key_order)
    }

    PipelineStep::Set(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));

      if let Some(condition) = &config.condition {
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, condition)? {
          return Ok(());
        }
      }

      document.set(&full_path, &config.value)
    }

    PipelineStep::Insert(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));

      if let Some(condition) = &config.condition {
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, condition)? {
          return Ok(());
        }
      }

      document.insert_into(&full_path, &config.value, crate::InsertPosition::Last)
    }

    PipelineStep::Delete(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));

      if full_path.contains("[]") {
        let concrete_selectors = document.resolve_selectors(&full_path);

        for selector in concrete_selectors.into_iter().rev() {
          if let Some(condition) = &config.condition {
            let parent_path = selector.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

            if !document.evaluate_condition(parent_path, condition)? {
              continue;
            }
          }

          document.delete(&selector)?;
        }

        Ok(())
      } else {
        if let Some(condition) = &config.condition {
          let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

          if !document.evaluate_condition(parent_path, condition)? {
            return Ok(());
          }
        }

        document.delete(&full_path)
      }
    }

    PipelineStep::Rename(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.from));

      if let Some(condition) = &config.condition {
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, condition)? {
          return Ok(());
        }
      }

      document.rename(&full_path, &config.to)
    }

    PipelineStep::Remove(config) => {
      let full_path = resolve_step_path(base_path, Some(&config.path));

      if let Some(condition) = &config.condition {
        let parent_path = full_path.rsplit_once('.').map(|(parent, _)| parent).unwrap_or("");

        if !document.evaluate_condition(parent_path, condition)? {
          return Ok(());
        }
      }

      document.remove(&full_path, &config.value)
    }

    PipelineStep::BlankLines(config) => {
      let full_path = resolve_step_path(base_path, config.path.as_deref());

      document.enforce_blank_lines(&full_path, config.count)
    }

    PipelineStep::Sort(config) => {
      let full_path = resolve_step_path(base_path, config.path.as_deref());
      let sort_fields = config.by.as_deref().map(crate::SortField::parse_list).unwrap_or_default();

      document.sort_items(&full_path, &sort_fields, config.case_sensitive)
    }

    PipelineStep::Directives(config) => {
      if config.ensure && config.remove {
        return Err(YerbaError::ParseError("directives: ensure and remove are mutually exclusive".to_string()));
      }

      if let Some(max) = config.max {
        let locations = document.directive_locations();

        if locations.len() > max {
          let positions: Vec<String> = locations.iter().map(|(line, col)| format!("{}:{}", line, col)).collect();

          return Err(YerbaError::ParseError(format!(
            "found {} directive markers (---) at {}, expected at most {}",
            locations.len(),
            positions.join(", "),
            max
          )));
        }
      }

      if config.ensure {
        document.ensure_directives()
      } else if config.remove {
        document.remove_directives()
      } else {
        Ok(())
      }
    }

    PipelineStep::Schema(config) => {
      let schema_path = yerbafile.resolve_path(&config.file);
      let schema = crate::schema::load_schema(&schema_path)?;
      let selector = resolve_step_path(base_path, config.path.as_deref());
      let errors = document.validate_schema(&schema, config.items, if selector.is_empty() { None } else { Some(&selector) });

      if errors.is_empty() {
        Ok(())
      } else {
        Err(YerbaError::SchemaValidation(errors))
      }
    }

    PipelineStep::Unique(config) => {
      let full_path = resolve_step_path(base_path, config.path.as_deref());
      let duplicates = document.unique_with_options(&full_path, &config.by, config.remove, config.allow_blank_duplicates)?;

      if !duplicates.is_empty() && !config.remove {
        return Err(YerbaError::DuplicateValues(duplicates));
      }

      Ok(())
    }

    PipelineStep::FinalNewline(config) => document.enforce_final_newline(config.count),
  }
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
