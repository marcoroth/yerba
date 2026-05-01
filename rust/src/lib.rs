mod document;
mod quote_style;
pub mod yerbafile;

pub use document::{Document, YerbaError};
pub use quote_style::QuoteStyle;
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
