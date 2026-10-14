use rowan::ast::AstNode;
use rowan::{TextRange, TextSize};

use yaml_parser::ast::{BlockMap, BlockMapEntry, BlockSeq, FlowMap, FlowMapEntry, FlowSeq};
use yaml_parser::{SyntaxKind, SyntaxNode, SyntaxToken};

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarValue {
  pub text: String,
  pub kind: SyntaxKind,
  pub file_path: Option<String>,
  pub selector: Option<String>,
  pub line: Option<usize>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YerbaValueType {
  Null = 0,
  Boolean = 1,
  Integer = 2,
  Float = 3,
  String = 4,
}

pub fn detect_yaml_type(scalar: &ScalarValue) -> YerbaValueType {
  resolve_scalar(scalar).value_type()
}

pub fn raw_scalar_value(token: &SyntaxToken) -> Option<String> {
  match token.kind() {
    SyntaxKind::PLAIN_SCALAR => Some(fold_flow_scalar(token.text())),

    SyntaxKind::DOUBLE_QUOTED_SCALAR => {
      let text = token.text();
      Some(unescape_double_quoted(&text[1..text.len() - 1]))
    }

    SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let text = token.text();
      Some(unescape_single_quoted(&text[1..text.len() - 1]))
    }

    _ => None,
  }
}

pub fn extract_scalar(node: &SyntaxNode) -> Option<ScalarValue> {
  if let Some(token) = find_scalar_token(node) {
    return raw_scalar_value(&token).map(|text| ScalarValue {
      text,
      kind: token.kind(),
      file_path: None,
      selector: None,
      line: None,
    });
  }

  let block_token = node
    .descendants_with_tokens()
    .filter_map(|element| element.into_token())
    .find(|token| token.kind() == SyntaxKind::BLOCK_SCALAR_TEXT)?;

  let dedented = dedent_block_scalar(block_token.text());

  let folded = node
    .descendants()
    .find(|descendant| descendant.kind() == SyntaxKind::BLOCK_SCALAR)
    .map(|scalar| scalar.text().to_string().trim_start().starts_with('>'))
    .unwrap_or(false);

  Some(ScalarValue {
    text: if folded { fold_flow_scalar(&dedented) } else { dedented },
    kind: SyntaxKind::BLOCK_SCALAR_TEXT,
    file_path: None,
    selector: None,
    line: None,
  })
}

pub fn find_block_map(node: &SyntaxNode) -> Option<BlockMap> {
  node.descendants().find_map(BlockMap::cast)
}

pub fn find_block_sequence(node: &SyntaxNode) -> Option<BlockSeq> {
  node.descendants().find_map(BlockSeq::cast)
}

fn first_collection_node(node: &SyntaxNode) -> Option<SyntaxNode> {
  node.descendants().find(|descendant| {
    matches!(
      descendant.kind(),
      SyntaxKind::BLOCK_MAP | SyntaxKind::BLOCK_SEQ | SyntaxKind::FLOW_MAP | SyntaxKind::FLOW_SEQ
    )
  })
}

pub fn find_flow_sequence(node: &SyntaxNode) -> Option<FlowSeq> {
  first_collection_node(node).and_then(FlowSeq::cast)
}

pub fn find_flow_map(node: &SyntaxNode) -> Option<FlowMap> {
  first_collection_node(node).and_then(FlowMap::cast)
}

pub fn flow_sequence_entries(sequence: &FlowSeq) -> Vec<SyntaxNode> {
  sequence
    .entries()
    .map(|entries| entries.entries().map(|entry| entry.syntax().clone()).collect())
    .unwrap_or_default()
}

pub fn flow_map_entries(map: &FlowMap) -> Vec<FlowMapEntry> {
  map.entries().map(|entries| entries.entries().collect()).unwrap_or_default()
}

pub fn find_flow_entry_by_key(map: &FlowMap, key: &str) -> Option<FlowMapEntry> {
  flow_map_entries(map)
    .into_iter()
    .find(|entry| entry.key().and_then(|found| extract_scalar_text(found.syntax())).as_deref() == Some(key))
}

pub fn in_flow_collection(node: &SyntaxNode) -> bool {
  node
    .ancestors()
    .any(|ancestor| matches!(ancestor.kind(), SyntaxKind::FLOW_MAP | SyntaxKind::FLOW_SEQ))
}

pub enum FirstCollection {
  Map(BlockMap),
  Sequence(BlockSeq),
}

pub fn first_collection(node: &SyntaxNode) -> Option<FirstCollection> {
  match (find_block_map(node), find_block_sequence(node)) {
    (Some(map), Some(sequence)) => {
      if sequence.syntax().text_range().start() <= map.syntax().text_range().start() {
        Some(FirstCollection::Sequence(sequence))
      } else {
        Some(FirstCollection::Map(map))
      }
    }

    (Some(map), None) => Some(FirstCollection::Map(map)),
    (None, Some(sequence)) => Some(FirstCollection::Sequence(sequence)),
    (None, None) => None,
  }
}

pub fn is_map_key(token: &SyntaxToken) -> bool {
  token.parent_ancestors().any(|ancestor| ancestor.kind() == SyntaxKind::BLOCK_MAP_KEY)
}

pub fn find_entry_by_key(map: &BlockMap, key: &str) -> Option<BlockMapEntry> {
  map.entries().find(|entry| {
    entry
      .key()
      .and_then(|key_node| extract_scalar_text(key_node.syntax()))
      .map(|key_text| key_text == key)
      .unwrap_or(false)
  })
}

pub fn find_scalar_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node.descendants_with_tokens().filter_map(|element| element.into_token()).find(|token| {
    matches!(
      token.kind(),
      SyntaxKind::PLAIN_SCALAR | SyntaxKind::DOUBLE_QUOTED_SCALAR | SyntaxKind::SINGLE_QUOTED_SCALAR
    )
  })
}

pub fn format_scalar_value(value: &str, kind: SyntaxKind) -> String {
  match kind {
    SyntaxKind::DOUBLE_QUOTED_SCALAR => format!("\"{}\"", escape_double_quoted(value)),

    SyntaxKind::SINGLE_QUOTED_SCALAR => {
      let escaped = value.replace('\'', "''");
      format!("'{}'", escaped)
    }

    _ => value.to_string(),
  }
}

pub fn is_control_character(character: char) -> bool {
  matches!(character as u32, 0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f | 0x7f | 0x80..=0x84 | 0x86..=0x9f)
}

pub fn is_single_quotable(value: &str) -> bool {
  !value.contains('\n') && !value.contains('\r') && !value.chars().any(is_control_character)
}

pub fn escape_double_quoted(value: &str) -> String {
  let mut result = String::with_capacity(value.len());

  for character in value.chars() {
    match character {
      '\\' => result.push_str("\\\\"),
      '"' => result.push_str("\\\""),
      '\n' => result.push_str("\\n"),
      '\r' => result.push_str("\\r"),
      '\t' => result.push_str("\\t"),
      _ if is_control_character(character) => result.push_str(&format!("\\x{:02x}", character as u32)),
      _ => result.push(character),
    }
  }

  result
}

const LEADING_INDICATORS: [char; 16] = ['#', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`', ',', '[', ']', '{', '}'];
const FLOW_INDICATORS: [char; 5] = [',', '[', ']', '{', '}'];

pub fn is_plain_safe(value: &str) -> bool {
  if value.is_empty() || value != value.trim() {
    return false;
  }

  if value.contains('\n') || value.contains('\t') {
    return false;
  }

  if value.chars().any(is_control_character) {
    return false;
  }

  if value.contains(": ") || value.ends_with(':') {
    return false;
  }

  if value.contains(" #") {
    return false;
  }

  if value.starts_with("---") || value.starts_with("...") {
    return false;
  }

  let mut characters = value.chars();
  let first = characters.next().expect("value is non-empty");

  if LEADING_INDICATORS.contains(&first) {
    return false;
  }

  if matches!(first, '-' | '?' | ':') {
    return matches!(characters.next(), Some(next) if next != ' ');
  }

  true
}

pub fn is_plain_safe_in_flow(value: &str) -> bool {
  is_plain_safe(value) && !value.contains(FLOW_INDICATORS)
}

pub fn is_quoted_scalar(value: &str) -> bool {
  let bytes = value.as_bytes();

  if bytes.len() < 2 {
    return false;
  }

  match (bytes[0], bytes[bytes.len() - 1]) {
    (b'"', b'"') => {
      let interior = &value[1..value.len() - 1];
      let mut escaped = false;

      for character in interior.chars() {
        if escaped {
          escaped = false;
          continue;
        }

        match character {
          '\\' => escaped = true,
          '"' => return false,
          _ => {}
        }
      }

      !escaped
    }

    (b'\'', b'\'') => {
      let interior = &value[1..value.len() - 1];
      let mut characters = interior.chars().peekable();

      while let Some(character) = characters.next() {
        if character == '\'' {
          if characters.peek() == Some(&'\'') {
            characters.next();
          } else {
            return false;
          }
        }
      }

      true
    }

    _ => false,
  }
}

pub fn is_flow_collection(value: &str) -> bool {
  (value.starts_with('[') && value.ends_with(']')) || (value.starts_with('{') && value.ends_with('}'))
}

pub fn is_inline_scalar_safe(value: &str) -> bool {
  if value.is_empty() {
    return true;
  }

  if value.contains('\n') || value.starts_with("- ") {
    return false;
  }

  is_quoted_scalar(value) || is_flow_collection(value) || is_plain_safe(value)
}

pub fn needs_quoting(value: &str) -> bool {
  is_yaml_non_string(value) || resolves_to_non_string_in_psych(value) || !is_plain_safe(value)
}

pub fn needs_quoting_in_flow(value: &str) -> bool {
  is_yaml_non_string(value) || resolves_to_non_string_in_psych(value) || !is_plain_safe_in_flow(value)
}

/// Psych loads dates, times and symbols as `Date`, `Time` and `Symbol`, which Yerba reads as strings,
/// and raises on float-shaped values like `.e+5`, so those need quoting even though they read as strings.
pub fn resolves_to_non_string_in_psych(value: &str) -> bool {
  !matches!(resolve_plain_scalar(value), PlainScalar::String)
    || is_psych_float(value)
    || is_psych_date_or_time(value)
    || (value.len() > 1 && value.starts_with(':'))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlainScalar {
  Null,
  Boolean(bool),
  Integer(i128),
  Float(f64),
  String,
}

impl PlainScalar {
  pub fn value_type(self) -> YerbaValueType {
    match self {
      PlainScalar::Null => YerbaValueType::Null,
      PlainScalar::Boolean(_) => YerbaValueType::Boolean,
      PlainScalar::Integer(_) => YerbaValueType::Integer,
      PlainScalar::Float(_) => YerbaValueType::Float,
      PlainScalar::String => YerbaValueType::String,
    }
  }
}

pub fn resolve_scalar(scalar: &ScalarValue) -> PlainScalar {
  if scalar.kind != SyntaxKind::PLAIN_SCALAR {
    return PlainScalar::String;
  }

  resolve_plain_scalar(&scalar.text)
}

/// Resolves a plain scalar the way Psych's `ScalarScanner#tokenize` does (YAML 1.1 with Psych's
/// extensions, legacy integers included), except that dates, times and symbols stay strings.
pub fn resolve_plain_scalar(value: &str) -> PlainScalar {
  if value.is_empty() {
    return PlainScalar::Null;
  }

  if value.len() <= 5 {
    let is = |word: &str| value.eq_ignore_ascii_case(word);

    if is("~") || is("null") {
      return PlainScalar::Null;
    }

    if is("yes") || is("true") || is("on") {
      return PlainScalar::Boolean(true);
    }

    if is("no") || is("false") || is("off") {
      return PlainScalar::Boolean(false);
    }

    if is(".inf") || is("+.inf") {
      return PlainScalar::Float(f64::INFINITY);
    }

    if is("-.inf") {
      return PlainScalar::Float(f64::NEG_INFINITY);
    }

    if is(".nan") {
      return PlainScalar::Float(f64::NAN);
    }
  }

  if is_psych_sexagesimal(value) {
    return sexagesimal_value(value);
  }

  if is_psych_float(value) {
    let mut cleaned: String = value.chars().filter(|character| !matches!(character, ',' | '_')).collect();

    if let Some(dot) = cleaned.find('.') {
      if matches!(cleaned.as_bytes().get(dot + 1), None | Some(b'e') | Some(b'E')) {
        cleaned.remove(dot);
      }
    }

    return cleaned.parse::<f64>().map(PlainScalar::Float).unwrap_or(PlainScalar::String);
  }

  if is_psych_integer(value) {
    return integer_value(value);
  }

  PlainScalar::String
}

fn integer_value(value: &str) -> PlainScalar {
  let cleaned: String = value.chars().filter(|character| !matches!(character, ',' | '_')).collect();

  let (negative, digits) = match cleaned.as_bytes()[0] {
    b'-' => (true, &cleaned[1..]),
    b'+' => (false, &cleaned[1..]),
    _ => (false, cleaned.as_str()),
  };

  let (radix, digits) = if let Some(digits) = digits.strip_prefix("0b") {
    (2, digits)
  } else if let Some(digits) = digits.strip_prefix("0x") {
    (16, digits)
  } else if digits.len() > 1 && digits.starts_with('0') {
    (8, &digits[1..])
  } else {
    (10, digits)
  };

  let magnitude = u128::from_str_radix(digits, radix).ok().and_then(|magnitude| i128::try_from(magnitude).ok());
  let integer = magnitude.map(|magnitude| if negative { -magnitude } else { magnitude });

  integer_or_float(integer, || {
    let magnitude = match radix {
      10 => digits.parse::<f64>().unwrap_or(f64::INFINITY),
      _ => digits
        .chars()
        .fold(0.0, |total, digit| total * f64::from(radix) + f64::from(digit.to_digit(radix).unwrap_or(0))),
    };

    if negative {
      -magnitude
    } else {
      magnitude
    }
  })
}

fn integer_or_float(integer: Option<i128>, approximate: impl FnOnce() -> f64) -> PlainScalar {
  match integer {
    Some(integer) if i64::try_from(integer).is_ok() || u64::try_from(integer).is_ok() => PlainScalar::Integer(integer),
    _ => PlainScalar::Float(approximate()),
  }
}

/// Psych sums the parts as `part * 60 ** (index - 2).abs`, so `1:30` is 5400, not 90.
fn sexagesimal_value(value: &str) -> PlainScalar {
  let parts: Vec<&str> = value.split(':').collect();
  let exponent = |index: usize| (index as i32 - 2).unsigned_abs();

  let float_total = || {
    parts.iter().enumerate().fold(0.0, |total, (index, part)| {
      total + ruby_number_prefix(part, true).parse::<f64>().unwrap_or(0.0) * 60f64.powi(exponent(index) as i32)
    })
  };

  if value.contains('.') {
    return PlainScalar::Float(float_total());
  }

  let integer_total = parts.iter().enumerate().try_fold(0i128, |total, (index, part)| {
    let part = ruby_number_prefix(part, false).parse::<i128>().ok()?;

    total.checked_add(part.checked_mul(60i128.pow(exponent(index)))?)
  });

  integer_or_float(integer_total, float_total)
}

/// The leading number Ruby's `String#to_i` (or `#to_f` with `fraction`) reads, without underscores.
fn ruby_number_prefix(text: &str, fraction: bool) -> String {
  let bytes = text.as_bytes();
  let mut number = String::new();
  let mut position = skip_sign(bytes);

  if position == 1 {
    number.push(bytes[0] as char);
  }

  position = push_ruby_digits(bytes, position, &mut number);

  if fraction && bytes.get(position) == Some(&b'.') && bytes.get(position + 1).is_some_and(u8::is_ascii_digit) {
    number.push('.');
    push_ruby_digits(bytes, position + 1, &mut number);
  }

  number
}

fn push_ruby_digits(bytes: &[u8], mut position: usize, number: &mut String) -> usize {
  let start = position;

  while let Some(&byte) = bytes.get(position) {
    let underscore_between_digits =
      byte == b'_' && position > start && bytes[position - 1].is_ascii_digit() && bytes.get(position + 1).is_some_and(u8::is_ascii_digit);

    if byte.is_ascii_digit() {
      number.push(byte as char);
    } else if !underscore_between_digits {
      break;
    }

    position += 1;
  }

  position
}

pub fn is_yaml_1_2_core_non_string(value: &str) -> bool {
  if matches!(
    value,
    "" | "~" | "null" | "Null" | "NULL" | "true" | "True" | "TRUE" | "false" | "False" | "FALSE" | ".nan" | ".NaN" | ".NAN"
  ) {
    return true;
  }

  let bytes = value.as_bytes();
  let unsigned = &bytes[skip_sign(bytes)..];

  if matches!(unsigned, b".inf" | b".Inf" | b".INF") {
    return true;
  }

  let all = |digits: &[u8], valid: fn(&u8) -> bool| !digits.is_empty() && digits.iter().all(valid);

  match bytes {
    [b'0', b'o', digits @ ..] if all(digits, |byte| matches!(byte, b'0'..=b'7')) => return true,
    [b'0', b'x', digits @ ..] if all(digits, u8::is_ascii_hexdigit) => return true,
    _ => {}
  }

  let integer = unsigned.iter().take_while(|byte| byte.is_ascii_digit()).count();
  let mut position = integer;

  if unsigned.get(position) == Some(&b'.') {
    let fraction = unsigned[position + 1..].iter().take_while(|byte| byte.is_ascii_digit()).count();

    if integer == 0 && fraction == 0 {
      return false;
    }

    position += 1 + fraction;
  } else if integer == 0 {
    return false;
  }

  if matches!(unsigned.get(position), Some(b'e') | Some(b'E')) {
    let exponent_start = position + 1 + usize::from(matches!(unsigned.get(position + 1), Some(b'+') | Some(b'-')));
    let exponent = unsigned
      .get(exponent_start..)
      .unwrap_or_default()
      .iter()
      .take_while(|byte| byte.is_ascii_digit())
      .count();

    if exponent == 0 {
      return false;
    }

    position = exponent_start + exponent;
  }

  position == unsigned.len()
}

fn take_digits(bytes: &[u8], mut position: usize, min: usize, max: usize) -> Option<usize> {
  let start = position;

  while position < bytes.len() && position - start < max && bytes[position].is_ascii_digit() {
    position += 1;
  }

  (position - start >= min).then_some(position)
}

fn skip_sign(bytes: &[u8]) -> usize {
  usize::from(matches!(bytes.first(), Some(b'+') | Some(b'-')))
}

fn is_psych_date_or_time(value: &str) -> bool {
  let bytes = value.as_bytes();
  let start = usize::from(bytes.first() == Some(&b'-'));

  let Some(position) = take_digits(bytes, start, 4, 4) else { return false };
  if bytes.get(position) != Some(&b'-') {
    return false;
  }

  let Some(position) = take_digits(bytes, position + 1, 1, 2) else {
    return false;
  };
  if bytes.get(position) != Some(&b'-') {
    return false;
  }

  let Some(position) = take_digits(bytes, position + 1, 1, 2) else {
    return false;
  };

  match bytes.get(position) {
    None => true,
    Some(byte) => matches!(byte, b'T' | b't') || byte.is_ascii_whitespace(),
  }
}

fn is_psych_sexagesimal(value: &str) -> bool {
  let bytes = value.as_bytes();
  let mut position = skip_sign(bytes);

  if !bytes.get(position).is_some_and(u8::is_ascii_digit) {
    return false;
  }

  while bytes.get(position).is_some_and(|byte| byte.is_ascii_digit() || *byte == b'_') {
    position += 1;
  }

  let mut groups = 0;

  while bytes.get(position) == Some(&b':') && groups < 2 {
    let Some(next) = take_digits(bytes, position + 1, 1, 2) else { return false };

    if next - position == 3 && bytes[position + 1] > b'5' {
      return false;
    }

    position = next;
    groups += 1;
  }

  if groups == 0 {
    return false;
  }

  if bytes.get(position) == Some(&b'.') {
    position += 1;

    while bytes.get(position).is_some_and(|byte| byte.is_ascii_digit() || *byte == b'_') {
      position += 1;
    }
  }

  position == bytes.len()
}

fn is_psych_float(value: &str) -> bool {
  let bytes = value.as_bytes();
  let sign = skip_sign(bytes);
  let mut position = sign;

  if bytes.get(position).is_some_and(u8::is_ascii_digit) {
    while bytes.get(position).is_some_and(|byte| byte.is_ascii_digit() || *byte == b'_' || *byte == b',') {
      position += 1;
    }
  }

  if bytes.get(position) != Some(&b'.') {
    return false;
  }

  position += 1;

  while bytes.get(position).is_some_and(u8::is_ascii_digit) {
    position += 1;
  }

  if matches!(bytes.get(position), Some(b'e') | Some(b'E')) {
    if !matches!(bytes.get(position + 1), Some(b'+') | Some(b'-')) {
      return false;
    }

    let Some(next) = take_digits(bytes, position + 2, 1, usize::MAX) else {
      return false;
    };
    position = next;
  }

  position == bytes.len() && bytes.len() > sign + 1
}

fn is_psych_integer(value: &str) -> bool {
  let bytes = value.as_bytes();
  let rest = &bytes[skip_sign(bytes)..];

  let prefixed = |digits: &[u8], valid: fn(&u8) -> bool| {
    let digits: Vec<&u8> = digits.iter().skip_while(|byte| matches!(byte, b'_' | b',')).collect();

    digits.first().is_some_and(|byte| valid(byte)) && digits.iter().all(|byte| valid(byte) || matches!(byte, b'_' | b','))
  };

  match rest {
    [] => false,
    [b'0'] => true,
    [b'0', b'b', digits @ ..] => prefixed(digits, |byte| matches!(byte, b'0' | b'1')),
    [b'0', b'x', digits @ ..] => prefixed(digits, u8::is_ascii_hexdigit),
    [b'0', digits @ ..] => prefixed(digits, |byte| matches!(byte, b'0'..=b'7')),
    [first, tail @ ..] if (b'1'..=b'9').contains(first) => {
      let mut tail = tail.iter().peekable();

      while let Some(byte) = tail.next() {
        match byte {
          b'0'..=b'9' => {}
          b'_' | b',' if tail.peek().is_some_and(|next| next.is_ascii_digit()) => {}
          _ => return false,
        }
      }

      true
    }
    _ => false,
  }
}

pub fn quote_scalar(value: &str) -> String {
  if needs_quoting(value) {
    format_scalar_value(value, SyntaxKind::DOUBLE_QUOTED_SCALAR)
  } else {
    value.to_string()
  }
}

pub fn quote_scalar_styled(value: &str, style: Option<&str>) -> String {
  match style {
    Some("double") => format_scalar_value(value, SyntaxKind::DOUBLE_QUOTED_SCALAR),
    Some("single") if is_single_quotable(value) => format_scalar_value(value, SyntaxKind::SINGLE_QUOTED_SCALAR),
    Some("flow") if needs_quoting_in_flow(value) => format_scalar_value(value, SyntaxKind::DOUBLE_QUOTED_SCALAR),
    Some("flow") => value.to_string(),

    _ => quote_scalar(value),
  }
}

pub fn extract_scalar_text(node: &SyntaxNode) -> Option<String> {
  extract_scalar(node).map(|scalar| scalar.text)
}

pub fn dedent_block_scalar(text: &str) -> String {
  let lines: Vec<&str> = text.lines().collect();
  let min_indent = lines
    .iter()
    .filter(|line| !line.trim().is_empty())
    .map(|line| line.len() - line.trim_start().len())
    .min()
    .unwrap_or(0);

  let dedented: String = lines
    .iter()
    .map(|line| if line.len() >= min_indent { &line[min_indent..] } else { line.trim() })
    .collect::<Vec<_>>()
    .join("\n");

  dedented.trim().to_string()
}

fn push_hex_escape(result: &mut String, characters: &mut Scan<'_>, digits: usize) {
  let escape: String = characters.clone().take(digits).collect();

  let decoded = (escape.len() == digits && escape.chars().all(|character| character.is_ascii_hexdigit()))
    .then(|| u32::from_str_radix(&escape, 16).ok().and_then(char::from_u32))
    .flatten();

  match decoded {
    Some(character) => {
      for _ in 0..digits {
        characters.next();
      }

      result.push(character);
    }

    None => {
      result.push('\\');
      result.push(match digits {
        2 => 'x',
        4 => 'u',
        _ => 'U',
      });
    }
  }
}

type Scan<'a> = std::iter::Peekable<std::str::Chars<'a>>;

fn skip_spaces(characters: &mut Scan<'_>) {
  while matches!(characters.peek(), Some(' ' | '\t')) {
    characters.next();
  }
}

fn fold_break(result: &mut String, characters: &mut Scan<'_>) {
  while result.ends_with(' ') || result.ends_with('\t') {
    result.pop();
  }

  let mut breaks = 1;

  loop {
    skip_spaces(characters);

    match characters.peek() {
      Some('\n') => {
        characters.next();
        breaks += 1;
      }

      Some('\r') => {
        characters.next();
        characters.next_if_eq(&'\n');
        breaks += 1;
      }

      _ => break,
    }
  }

  if breaks == 1 {
    result.push(' ');
  } else {
    result.extend(std::iter::repeat_n('\n', breaks - 1));
  }
}

pub fn fold_flow_scalar(text: &str) -> String {
  if !text.contains('\n') && !text.contains('\r') {
    return text.to_string();
  }

  let mut result = String::with_capacity(text.len());
  let mut characters = text.chars().peekable();

  while let Some(character) = characters.next() {
    match character {
      '\n' => fold_break(&mut result, &mut characters),

      '\r' => {
        characters.next_if_eq(&'\n');
        fold_break(&mut result, &mut characters);
      }

      _ => result.push(character),
    }
  }

  result
}

pub fn unescape_double_quoted(text: &str) -> String {
  let mut result = String::with_capacity(text.len());
  let mut chars = text.chars().peekable();

  while let Some(character) = chars.next() {
    if character == '\n' {
      fold_break(&mut result, &mut chars);
      continue;
    }

    if character == '\r' {
      chars.next_if_eq(&'\n');
      fold_break(&mut result, &mut chars);
      continue;
    }

    if character == '\\' {
      match chars.next() {
        Some('n') => result.push('\n'),
        Some('t') => result.push('\t'),
        Some('r') => result.push('\r'),
        Some('\\') => result.push('\\'),
        Some('"') => result.push('"'),
        Some('/') => result.push('/'),
        Some('0') => result.push('\0'),
        Some('a') => result.push('\u{07}'),
        Some('b') => result.push('\u{08}'),
        Some('e') => result.push('\u{1b}'),
        Some('v') => result.push('\u{0b}'),
        Some(' ') => result.push(' '),
        Some('_') => result.push('\u{a0}'),
        Some('N') => result.push('\u{85}'),
        Some('L') => result.push('\u{2028}'),
        Some('P') => result.push('\u{2029}'),
        Some('x') => push_hex_escape(&mut result, &mut chars, 2),
        Some('u') => push_hex_escape(&mut result, &mut chars, 4),
        Some('U') => push_hex_escape(&mut result, &mut chars, 8),
        Some('\n') => skip_spaces(&mut chars),

        Some('\r') => {
          chars.next_if_eq(&'\n');
          skip_spaces(&mut chars);
        }

        Some(other) => {
          result.push('\\');
          result.push(other);
        }
        None => result.push('\\'),
      }
    } else {
      result.push(character);
    }
  }

  result
}

pub fn unescape_single_quoted(text: &str) -> String {
  fold_flow_scalar(&text.replace("''", "'"))
}

pub fn line_at(source: &str, offset: usize) -> usize {
  source[..offset].matches('\n').count() + 1
}

pub fn line_start_at(source: &str, offset: usize) -> usize {
  source[..offset].rfind('\n').map(|position| position + 1).unwrap_or(0)
}

pub fn column_at(source: &str, offset: usize) -> usize {
  offset - line_start_at(source, offset)
}

#[derive(Debug)]
pub struct LineIndex {
  source: String,
  line_starts: Vec<usize>,
  length: usize,
}

impl LineIndex {
  pub fn new(source: &str) -> Self {
    let line_starts = std::iter::once(0).chain(source.match_indices('\n').map(|(position, _)| position + 1)).collect();

    Self {
      source: source.to_string(),
      line_starts,
      length: source.len(),
    }
  }

  pub fn source_length(&self) -> usize {
    self.length
  }

  pub fn line_at(&self, offset: usize) -> usize {
    self.line_starts.partition_point(|&start| start <= offset)
  }

  pub fn column_at(&self, offset: usize) -> usize {
    offset - self.line_starts[self.line_at(offset) - 1]
  }

  pub fn character_column_at(&self, offset: usize) -> usize {
    let line_start = self.line_starts[self.line_at(offset) - 1];

    self.source.get(line_start..offset).map_or(offset - line_start, |text| text.chars().count())
  }
}

pub fn preceding_whitespace_indent(node: &SyntaxNode) -> String {
  if let Some(token) = preceding_whitespace_token(node) {
    let text = token.text();

    if let Some(newline) = text.rfind('\n') {
      return text[newline + 1..].to_string();
    }
  }

  let start_offset: usize = node.text_range().start().into();
  let root = node.ancestors().last().unwrap_or_else(|| node.clone());
  let source = root.text().to_string();

  if start_offset > 0 {
    let line_start = line_start_at(&source, start_offset);

    if line_start > 0 {
      return source[line_start..start_offset].to_string();
    }
  }

  String::new()
}

pub fn preceding_whitespace_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .prev_sibling_or_token()
    .and_then(|sibling| sibling.into_token())
    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
}

pub fn following_whitespace_token(node: &SyntaxNode) -> Option<SyntaxToken> {
  node
    .next_sibling_or_token()
    .and_then(|sibling| sibling.into_token())
    .filter(|token| token.kind() == SyntaxKind::WHITESPACE)
}

pub fn removal_range(node: &SyntaxNode) -> TextRange {
  let node_range = node.text_range();

  if let Some(whitespace_token) = preceding_whitespace_token(node) {
    let whitespace_text = whitespace_token.text();
    let whitespace_start = whitespace_token.text_range().start();

    let remove_from = whitespace_text
      .rfind('\n')
      .map(|offset| whitespace_start + TextSize::from(offset as u32))
      .unwrap_or(whitespace_start);

    return TextRange::new(remove_from, node_range.end());
  }

  if let Some(whitespace_token) = following_whitespace_token(node) {
    return TextRange::new(node_range.start(), whitespace_token.text_range().end());
  }

  node_range
}

pub fn is_yaml_non_string(value: &str) -> bool {
  detect_yaml_type_from_plain(value) != YerbaValueType::String || is_yaml_1_2_core_non_string(value)
}

pub fn detect_yaml_type_from_plain(value: &str) -> YerbaValueType {
  resolve_plain_scalar(value).value_type()
}
