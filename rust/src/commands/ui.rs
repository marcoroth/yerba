//! Terminal styling.
//!
//! Styles are declared once here and referred to by meaning, so a command asks
//! for `success` or `failure` rather than picking a colour. The palette is the
//! terminal's own sixteen, so output matches whatever theme is in use instead of
//! imposing one. Colour is dropped entirely when the output is piped or
//! `NO_COLOR` is set.

use std::io::IsTerminal;
use std::sync::OnceLock;

fn colored() -> bool {
  static COLORED: OnceLock<bool> = OnceLock::new();

  *COLORED.get_or_init(|| {
    if std::env::var_os("NO_COLOR").is_some() {
      return false;
    }

    if std::env::var_os("CLICOLOR_FORCE").is_some_and(|value| value != "0") {
      return true;
    }

    std::io::stderr().is_terminal() || super::is_github_actions()
  })
}

#[derive(Clone, Copy)]
pub struct Color(&'static str);

pub const GREEN: Color = Color("32");
pub const YELLOW: Color = Color("33");
pub const RED: Color = Color("31");
pub const CYAN: Color = Color("36");

pub struct Style {
  color: Option<Color>,
  bold: bool,
  dim: bool,
}

impl Style {
  pub const fn plain() -> Self {
    Style {
      color: None,
      bold: false,
      dim: false,
    }
  }

  pub const fn fg(color: Color) -> Self {
    Style {
      color: Some(color),
      bold: false,
      dim: false,
    }
  }

  pub const fn bold(mut self) -> Self {
    self.bold = true;
    self
  }

  pub const fn dim(mut self) -> Self {
    self.dim = true;
    self
  }

  pub fn paint(&self, text: impl std::fmt::Display) -> String {
    if !colored() {
      return text.to_string();
    }

    let mut codes: Vec<&str> = Vec::new();

    if self.bold {
      codes.push("1");
    }

    if self.dim {
      codes.push("2");
    }

    if let Some(color) = &self.color {
      codes.push(color.0);
    }

    if codes.is_empty() {
      return text.to_string();
    }

    format!("\x1b[{}m{}\x1b[0m", codes.join(";"), text)
  }
}

pub fn success(text: impl std::fmt::Display) -> String {
  Style::fg(GREEN).bold().paint(text)
}

pub fn pending(text: impl std::fmt::Display) -> String {
  Style::fg(YELLOW).bold().paint(text)
}

pub fn failure(text: impl std::fmt::Display) -> String {
  Style::fg(RED).bold().paint(text)
}

pub fn muted(text: impl std::fmt::Display) -> String {
  Style::fg(CYAN).paint(text)
}

pub fn subtle(text: impl std::fmt::Display) -> String {
  Style::plain().dim().paint(text)
}

pub fn strong(text: impl std::fmt::Display) -> String {
  Style::plain().bold().paint(text)
}

pub fn banner() -> String {
  format!("🧉 {} {}", Style::fg(GREEN).bold().paint("yerba"), subtle(format!("v{}", yerba::version())))
}

pub const INDENT: &str = "   ";
pub const STATUS_INDENT: &str = "  ";

pub mod glyph {
  pub const OK: &str = "✓";
  pub const PENDING: &str = "~";
  pub const FAIL: &str = "✗";
}
