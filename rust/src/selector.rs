/// A parsed selector used across selectors, conditions, and select fields.
///
/// Selectors starting with `.` are relative to the current item context.
/// All other selectors are absolute from the document root.
///
/// Examples:
///   ".title"              → Relative([Key("title")])
///   ".speakers[].name"    → Relative([Key("speakers"), AllItems, Key("name")])
///   "database.host"       → Absolute([Key("database"), Key("host")])
///   "[].title"            → Absolute([AllItems, Key("title")])
///   "[0].speakers[]"      → Absolute([Index(0), Key("speakers"), AllItems])
///   "[]"                  → Absolute([AllItems])
///
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
  Relative(Vec<SelectorSegment>),
  Absolute(Vec<SelectorSegment>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectorSegment {
  Key(String),
  AllItems,
  AllKeys,
  Index(usize),
}

impl Selector {
  pub fn parse(input: &str) -> Self {
    let input = input.trim();

    if input.is_empty() {
      return Selector::Absolute(Vec::new());
    }

    if let Some(rest) = input.strip_prefix('.') {
      Selector::Relative(parse_segments(rest))
    } else {
      Selector::Absolute(parse_segments(input))
    }
  }

  pub fn is_relative(&self) -> bool {
    matches!(self, Selector::Relative(_))
  }

  pub fn is_absolute(&self) -> bool {
    matches!(self, Selector::Absolute(_))
  }

  pub fn segments(&self) -> &[SelectorSegment] {
    match self {
      Selector::Relative(segments) | Selector::Absolute(segments) => segments,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.segments().is_empty()
  }

  pub fn ends_with_bracket(&self) -> bool {
    matches!(self.segments().last(), Some(SelectorSegment::AllItems | SelectorSegment::Index(_)))
  }

  pub fn has_brackets(&self) -> bool {
    self
      .segments()
      .iter()
      .any(|s| matches!(s, SelectorSegment::AllItems | SelectorSegment::Index(_)))
  }

  pub fn has_wildcard(&self) -> bool {
    self
      .segments()
      .iter()
      .any(|s| matches!(s, SelectorSegment::AllItems | SelectorSegment::AllKeys))
  }

  /// Split into the container selector (up to and including the last []) and the remaining field selector.
  /// Used to separate "where to search" from "what to extract".
  ///
  /// "[].speakers[].name" → ("[].speakers[]", ".name")
  /// "[].title"           → ("[]", ".title")
  /// "[]"                 → ("[]", "")
  /// "database.host"      → ("", "database.host")
  pub fn split_at_last_bracket(&self) -> (Selector, Selector) {
    let segments = self.segments();

    let last_bracket = segments
      .iter()
      .rposition(|s| matches!(s, SelectorSegment::AllItems | SelectorSegment::Index(_)));

    match last_bracket {
      Some(position) => {
        let container = segments[..=position].to_vec();
        let field = segments[position + 1..].to_vec();

        (Selector::Absolute(container), Selector::Relative(field))
      }
      None => (Selector::Absolute(Vec::new()), Selector::Absolute(segments.to_vec())),
    }
  }

  /// Split at the first [] bracket — used for condition evaluation.
  /// The condition is evaluated on items at the first container level.
  ///
  /// "[].speakers[]"     → ("[],", ".speakers[]")
  /// "[].title"          → ("[]", ".title")
  /// "[]"                → ("[]", "")
  pub fn split_at_first_bracket(&self) -> (Selector, Selector) {
    let segments = self.segments();

    let first_bracket = segments.iter().position(|s| matches!(s, SelectorSegment::AllItems | SelectorSegment::Index(_)));

    match first_bracket {
      Some(position) => {
        let container = segments[..=position].to_vec();
        let rest = segments[position + 1..].to_vec();

        (Selector::Absolute(container), Selector::Relative(rest))
      }

      None => (Selector::Absolute(Vec::new()), Selector::Absolute(segments.to_vec())),
    }
  }

  /// Resolve a relative selector against a base absolute selector.
  ///
  /// ".title" + "[]"          → "[].title"
  /// ".name"  + "[].speakers[]" → "[].speakers[].name"
  /// If self is absolute, returns self unchanged.
  pub fn resolve_relative(&self, base: &Selector) -> Selector {
    if self.is_absolute() {
      return self.clone();
    }

    let mut segments = base.segments().to_vec();
    segments.extend_from_slice(self.segments());

    Selector::Absolute(segments)
  }

  pub fn to_selector_string(&self) -> String {
    let segments = self.segments();

    if segments.is_empty() {
      return String::new();
    }

    let mut result = String::new();

    for (index, segment) in segments.iter().enumerate() {
      match segment {
        SelectorSegment::Key(key) => {
          if index > 0 && !matches!(segments.get(index - 1), Some(SelectorSegment::Key(_))) {
            if !result.ends_with('.') {
              result.push('.');
            }
          } else if index > 0 {
            result.push('.');
          }

          result.push_str(key);
        }

        SelectorSegment::AllItems => {
          result.push_str("[]");
        }

        SelectorSegment::AllKeys => {
          if index > 0 && !result.ends_with('.') {
            result.push('.');
          }

          result.push('*');
        }

        SelectorSegment::Index(i) => {
          result.push_str(&format!("[{}]", i));
        }
      }
    }

    result
  }

  pub fn parent_path(&self) -> String {
    let segments = self.segments();

    if segments.len() <= 1 {
      return String::new();
    }

    let parent_segments = &segments[..segments.len() - 1];
    let parent = match self {
      Selector::Relative(_) => Selector::Relative(parent_segments.to_vec()),
      Selector::Absolute(_) => Selector::Absolute(parent_segments.to_vec()),
    };

    parent.to_selector_string()
  }
}

fn parse_segments(input: &str) -> Vec<SelectorSegment> {
  let mut segments = Vec::new();
  let mut rest = input;

  while !rest.is_empty() {
    if rest.starts_with('[') {
      if let Some(close) = rest.find(']') {
        let inner = &rest[1..close];

        if inner.is_empty() {
          segments.push(SelectorSegment::AllItems);
        } else if let Ok(index) = inner.parse::<usize>() {
          segments.push(SelectorSegment::Index(index));
        }

        rest = &rest[close + 1..];

        if rest.starts_with('.') {
          rest = &rest[1..];
        }
      } else {
        break;
      }
    } else {
      let dot_index = rest.find('.');
      let bracket_index = rest.find('[');

      let split_at = match (dot_index, bracket_index) {
        (Some(dot), Some(bracket)) => Some(dot.min(bracket)),
        (Some(dot), None) => Some(dot),
        (None, Some(bracket)) => Some(bracket),
        (None, None) => None,
      };

      match split_at {
        Some(index) => {
          let key = &rest[..index];

          if key == "*" {
            segments.push(SelectorSegment::AllKeys);
          } else if !key.is_empty() {
            segments.push(SelectorSegment::Key(key.to_string()));
          }

          rest = &rest[index..];

          if rest.starts_with('.') {
            rest = &rest[1..];
          }
        }
        None => {
          if rest == "*" {
            segments.push(SelectorSegment::AllKeys);
          } else if !rest.is_empty() {
            segments.push(SelectorSegment::Key(rest.to_string()));
          }

          break;
        }
      }
    }
  }

  segments
}
