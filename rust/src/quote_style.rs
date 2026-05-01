use yaml_parser::SyntaxKind;

#[derive(Debug, Clone, PartialEq)]
pub enum QuoteStyle {
  Plain,
  SingleQuoted,
  DoubleQuoted,
  BlockLiteral,
  BlockFolded,
}

impl std::str::FromStr for QuoteStyle {
  type Err = String;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    match string {
      "plain" => Ok(QuoteStyle::Plain),
      "single" => Ok(QuoteStyle::SingleQuoted),
      "double" => Ok(QuoteStyle::DoubleQuoted),
      "literal" | "block-literal" => Ok(QuoteStyle::BlockLiteral),
      "folded" | "block-folded" => Ok(QuoteStyle::BlockFolded),
      _ => Err(format!("unknown quote style: '{}'", string)),
    }
  }
}

impl QuoteStyle {
  pub(crate) fn to_syntax_kind(&self) -> SyntaxKind {
    match self {
      QuoteStyle::Plain => SyntaxKind::PLAIN_SCALAR,
      QuoteStyle::SingleQuoted => SyntaxKind::SINGLE_QUOTED_SCALAR,
      QuoteStyle::DoubleQuoted => SyntaxKind::DOUBLE_QUOTED_SCALAR,
      QuoteStyle::BlockLiteral => SyntaxKind::BLOCK_SCALAR_TEXT,
      QuoteStyle::BlockFolded => SyntaxKind::BLOCK_SCALAR_TEXT,
    }
  }
}
