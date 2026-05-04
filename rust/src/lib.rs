pub mod didyoumean;
mod document;
mod error;
pub mod ffi;
pub mod json;
mod quote_style;
pub mod selector;
mod syntax;
mod yaml_writer;
pub mod yerbafile;

pub use document::{collect_selectors, Document, InsertPosition, SortField};
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
