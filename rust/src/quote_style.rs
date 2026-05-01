use clap::ValueEnum;
use yaml_parser::SyntaxKind;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum QuoteStyle {
  Plain,
  #[value(alias = "single-quoted")]
  Single,
  #[value(alias = "double-quoted")]
  Double,
  #[value(alias = "block-literal")]
  Literal,
  #[value(alias = "block-folded")]
  Folded,
}

impl std::str::FromStr for QuoteStyle {
  type Err = String;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    match string {
      "plain" => Ok(QuoteStyle::Plain),
      "single" | "single-quoted" => Ok(QuoteStyle::Single),
      "double" | "double-quoted" => Ok(QuoteStyle::Double),
      "literal" | "block-literal" => Ok(QuoteStyle::Literal),
      "folded" | "block-folded" => Ok(QuoteStyle::Folded),
      _ => Err(format!("unknown quote style: '{}'", string)),
    }
  }
}

impl QuoteStyle {
  pub(crate) fn to_syntax_kind(&self) -> SyntaxKind {
    match self {
      QuoteStyle::Plain => SyntaxKind::PLAIN_SCALAR,
      QuoteStyle::Single => SyntaxKind::SINGLE_QUOTED_SCALAR,
      QuoteStyle::Double => SyntaxKind::DOUBLE_QUOTED_SCALAR,
      QuoteStyle::Literal => SyntaxKind::BLOCK_SCALAR_TEXT,
      QuoteStyle::Folded => SyntaxKind::BLOCK_SCALAR_TEXT,
    }
  }
}
