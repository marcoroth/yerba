use clap::ValueEnum;
use yaml_parser::SyntaxKind;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum KeyStyle {
  /// Unquoted key (host:)
  Plain,
  /// Single-quoted key ('host':)
  Single,
  /// Double-quoted key ("host":)
  Double,
}

impl std::str::FromStr for KeyStyle {
  type Err = String;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    match string {
      "plain" => Ok(KeyStyle::Plain),
      "single" | "single-quoted" => Ok(KeyStyle::Single),
      "double" | "double-quoted" => Ok(KeyStyle::Double),
      _ => Err(format!("unknown key style: '{}'. Valid options: plain, single, double", string)),
    }
  }
}

impl KeyStyle {
  pub(crate) fn to_syntax_kind(&self) -> SyntaxKind {
    match self {
      KeyStyle::Plain => SyntaxKind::PLAIN_SCALAR,
      KeyStyle::Single => SyntaxKind::SINGLE_QUOTED_SCALAR,
      KeyStyle::Double => SyntaxKind::DOUBLE_QUOTED_SCALAR,
    }
  }
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum QuoteStyle {
  /// Unquoted value (host: localhost)
  Plain,
  /// Single-quoted value (host: 'localhost')
  #[value(alias = "single-quoted")]
  Single,
  /// Double-quoted value (host: "localhost")
  #[value(alias = "double-quoted")]
  Double,
  /// Literal block scalar, strip trailing newline (|-)
  #[value(alias = "block-literal", alias = "|-")]
  Literal,
  /// Literal block scalar, keep one trailing newline (|)
  #[value(alias = "|")]
  LiteralClip,
  /// Literal block scalar, keep all trailing newlines (|+)
  #[value(alias = "|+")]
  LiteralKeep,
  /// Folded block scalar, strip trailing newline (>-)
  #[value(alias = "block-folded", alias = ">-")]
  Folded,
  /// Folded block scalar, keep one trailing newline (>)
  #[value(alias = ">")]
  FoldedClip,
  /// Folded block scalar, keep all trailing newlines (>+)
  #[value(alias = ">+")]
  FoldedKeep,
}

impl std::str::FromStr for QuoteStyle {
  type Err = String;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let normalized = string.replace('_', "-");

    match normalized.as_str() {
      "plain" => Ok(QuoteStyle::Plain),
      "single" | "single-quoted" => Ok(QuoteStyle::Single),
      "double" | "double-quoted" => Ok(QuoteStyle::Double),
      "literal" | "block-literal" | "|-" => Ok(QuoteStyle::Literal),
      "literal-clip" | "|" => Ok(QuoteStyle::LiteralClip),
      "literal-keep" | "|+" => Ok(QuoteStyle::LiteralKeep),
      "folded" | "block-folded" | ">-" => Ok(QuoteStyle::Folded),
      "folded-clip" | ">" => Ok(QuoteStyle::FoldedClip),
      "folded-keep" | ">+" => Ok(QuoteStyle::FoldedKeep),
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
      QuoteStyle::Literal | QuoteStyle::LiteralClip | QuoteStyle::LiteralKeep | QuoteStyle::Folded | QuoteStyle::FoldedClip | QuoteStyle::FoldedKeep => {
        SyntaxKind::BLOCK_SCALAR_TEXT
      }
    }
  }

  pub(crate) fn is_block_scalar(&self) -> bool {
    matches!(
      self,
      QuoteStyle::Literal | QuoteStyle::LiteralClip | QuoteStyle::LiteralKeep | QuoteStyle::Folded | QuoteStyle::FoldedClip | QuoteStyle::FoldedKeep
    )
  }

  pub(crate) fn block_header(&self) -> &'static str {
    match self {
      QuoteStyle::Literal => "|-",
      QuoteStyle::LiteralClip => "|",
      QuoteStyle::LiteralKeep => "|+",
      QuoteStyle::Folded => ">-",
      QuoteStyle::FoldedClip => ">",
      QuoteStyle::FoldedKeep => ">+",
      _ => "",
    }
  }
}
