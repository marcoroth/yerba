use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Document, QuoteStyle, YerbaError};

#[derive(Debug, Deserialize)]
pub struct Yerbafile {
  #[serde(default)]
  pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
  pub files: String,
  #[serde(default)]
  pub sort_keys: Option<SortKeysRule>,
  #[serde(default)]
  pub quote_style: Option<QuoteStyleRule>,
}

#[derive(Debug, Deserialize)]
pub struct SortKeysRule {
  #[serde(default)]
  pub path: String,
  pub order: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct QuoteStyleRule {
  #[serde(default = "default_key_style")]
  pub key_style: String,
  pub value_style: String,
  #[serde(default)]
  pub path: Option<String>,
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
    let yerbafile: Yerbafile = serde_yaml::from_str(&content)
      .map_err(|error| YerbaError::ParseError(format!("{}", error)))?;
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
      if let Some(sort_keys_rule) = &rule.sort_keys {
        let rule_path = sort_keys_rule.path.trim_end_matches("[]");
        let rule_path = rule_path.trim_end_matches('.');

        if rule_path != dot_path && !rule_path.is_empty() && !dot_path.is_empty() {
          continue;
        }

        if let Ok(pattern) = glob::Pattern::new(&rule.files) {
          if pattern.matches(file_path) || pattern.matches_path(std::path::Path::new(file_path)) {
            return Some(sort_keys_rule.order.clone());
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

      let file_strings: Vec<String> = files
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();

      if let Some(sort_keys_rule) = &rule.sort_keys {
        let key_order: Vec<&str> = sort_keys_rule
          .order
          .iter()
          .map(|key| key.as_str())
          .collect();

        let mut has_validation_error = false;

        for file in &file_strings {
          let document = match Document::parse_file(file) {
            Ok(document) => document,
            Err(error) => {
              results.push(RuleResult {
                file: file.clone(),
                changed: false,
                error: Some(format!("{}", error)),
              });

              has_validation_error = true;

              continue;
            }
          };

          if let Err(error) = document.validate_sort_keys(&sort_keys_rule.path, &key_order) {
            results.push(RuleResult {
              file: file.clone(),
              changed: false,
              error: Some(format!("{}", error)),
            });

            has_validation_error = true;
          }
        }

        if has_validation_error {
          continue;
        }
      }

      for file in &file_strings {
        let result = self.apply_rule_to_file(rule, file, write);
        results.push(result);
      }
    }

    results
  }

  fn apply_rule_to_file(&self, rule: &Rule, file: &str, write: bool) -> RuleResult {
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
    let mut had_error = None;

    if let Some(quote_style_rule) = &rule.quote_style {
      let dot_path = quote_style_rule.path.as_deref();

      match quote_style_rule.key_style.parse::<QuoteStyle>() {
        Ok(style) => {
          if let Err(error) = document.enforce_key_style(&style, dot_path) {
            had_error = Some(format!("{}", error));
          }
        }

        Err(error) => had_error = Some(error),
      }

      match quote_style_rule.value_style.parse::<QuoteStyle>() {
        Ok(style) => {
          if let Err(error) = document.enforce_quotes_at(&style, dot_path) {
            had_error = Some(format!("{}", error));
          }
        }

        Err(error) => had_error = Some(error),
      }
    }

    if let Some(sort_keys_rule) = &rule.sort_keys {
      let key_order: Vec<&str> = sort_keys_rule
        .order
        .iter()
        .map(|key| key.as_str())
        .collect();

      if let Err(error) = document.sort_keys(&sort_keys_rule.path, &key_order) {
        had_error = Some(format!("{}", error));
      }
    }

    let new_content = document.to_string();
    let changed = new_content != original;

    if changed && write {
      if let Err(error) = fs::write(file, &new_content) {
        had_error = Some(format!("{}", error));
      }
    }

    RuleResult {
      file: file.to_string(),
      changed,
      error: had_error,
    }
  }
}
