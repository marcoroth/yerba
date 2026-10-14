mod support;
use indoc::indoc;
use support::parse;
use yerba::{detect_yaml_type, needs_quoting, resolve_plain_scalar, PlainScalar, ScalarValue, YerbaValueType};

fn integer(value: &str) -> Option<i128> {
  match resolve_plain_scalar(value) {
    PlainScalar::Integer(integer) => Some(integer),
    _ => None,
  }
}

fn float(value: &str) -> Option<f64> {
  match resolve_plain_scalar(value) {
    PlainScalar::Float(float) => Some(float),
    _ => None,
  }
}

#[test]
fn test_octal_integers() {
  assert_eq!(integer("0755"), Some(493));
  assert_eq!(integer("-0755"), Some(-493));
  assert_eq!(integer("+0755"), Some(493));
  assert_eq!(integer("00"), Some(0));
  assert_eq!(integer("0_7"), Some(7));
  assert_eq!(integer("07_"), Some(7));
}

#[test]
fn test_binary_and_hex_integers() {
  assert_eq!(integer("0b1010"), Some(10));
  assert_eq!(integer("-0b101"), Some(-5));
  assert_eq!(integer("0b_1"), Some(1));
  assert_eq!(integer("0x1F"), Some(31));
  assert_eq!(integer("+0x1F"), Some(31));
  assert_eq!(integer("0x1F_"), Some(31));
}

#[test]
fn test_decimal_integers_with_separators() {
  assert_eq!(integer("1_000"), Some(1000));
  assert_eq!(integer("1,000"), Some(1000));
  assert_eq!(integer("-1_000"), Some(-1000));
  assert_eq!(integer("+12"), Some(12));
  assert_eq!(integer("-0"), Some(0));
}

#[test]
fn test_integers_beyond_i64_keep_unsigned_range() {
  assert_eq!(integer("9223372036854775808"), Some(9223372036854775808));
  assert_eq!(integer("18446744073709551615"), Some(18446744073709551615));
  assert_eq!(float("18446744073709551616"), Some(18446744073709551616.0));
}

#[test]
fn test_sexagesimal_integers() {
  assert_eq!(integer("12:30:00"), Some(45000));
  assert_eq!(integer("1:30"), Some(5400));
  assert_eq!(integer("-1:30"), Some(-1800));
  assert_eq!(integer("1:5"), Some(3900));
  assert_eq!(integer("1_0:30"), Some(37800));
}

#[test]
fn test_sexagesimal_floats() {
  assert_eq!(float("1:30.5"), Some(5430.0));
  assert_eq!(float("-1:30.5"), Some(-1770.0));
  assert_eq!(float("1:30."), Some(5400.0));
  assert_eq!(float("190:20:30.15"), Some(685230.15));
}

#[test]
fn test_floats() {
  assert_eq!(float("1.5"), Some(1.5));
  assert_eq!(float("1_000.5"), Some(1000.5));
  assert_eq!(float("1,000.5"), Some(1000.5));
  assert_eq!(float(".5"), Some(0.5));
  assert_eq!(float("-.5"), Some(-0.5));
  assert_eq!(float("1."), Some(1.0));
  assert_eq!(float("1.e+5"), Some(100000.0));
  assert_eq!(float("1.5E-3"), Some(0.0015));
}

#[test]
fn test_special_floats_in_any_case() {
  for value in [".inf", ".Inf", ".INF", ".iNf", "+.inf", "+.INF"] {
    assert_eq!(float(value), Some(f64::INFINITY), "{value}");
  }

  for value in ["-.inf", "-.Inf", "-.INF"] {
    assert_eq!(float(value), Some(f64::NEG_INFINITY), "{value}");
  }

  for value in [".nan", ".NaN", ".NAN", ".nAn"] {
    assert!(float(value).is_some_and(f64::is_nan), "{value}");
  }
}

#[test]
fn test_booleans_and_null_in_any_case() {
  for value in ["yes", "Yes", "yEs", "true", "tRuE", "on", "oN"] {
    assert_eq!(resolve_plain_scalar(value), PlainScalar::Boolean(true), "{value}");
  }

  for value in ["no", "NO", "false", "fAlSe", "off", "oFF"] {
    assert_eq!(resolve_plain_scalar(value), PlainScalar::Boolean(false), "{value}");
  }

  for value in ["", "~", "null", "Null", "nUlL"] {
    assert_eq!(resolve_plain_scalar(value), PlainScalar::Null, "{value}");
  }
}

#[test]
fn test_look_alikes_stay_strings() {
  for value in [
    "08",
    "09",
    "0o17",
    "0O17",
    "0X1F",
    "0B101",
    "0x",
    "1__0",
    "1_000_",
    "1,,0",
    "1:60",
    "12:3:45:00",
    "1.0e5",
    "1e5",
    "1E+5",
    ".",
    "-.",
    ".e+5",
    "1..5",
    "1.2.3",
    "v1.0",
    "0.1_5",
    "inf",
    "nan",
    "NaN",
    "Infinity",
    "y",
    "n",
    "yes!",
    "Yesss",
    "1a",
  ] {
    assert_eq!(resolve_plain_scalar(value), PlainScalar::String, "{value}");
  }
}

#[test]
fn test_dates_times_and_symbols_stay_strings() {
  for value in ["2025-01-01", "2025-1-1", "2025-01-01T10:00:00Z", "2025-01-01 10:00:00", ":foo", ":1"] {
    assert_eq!(resolve_plain_scalar(value), PlainScalar::String, "{value}");
  }
}

#[test]
fn test_detected_type_matches_resolved_value() {
  let plain = |text: &str| ScalarValue {
    text: text.to_string(),
    kind: yaml_parser::SyntaxKind::PLAIN_SCALAR,
    file_path: None,
    selector: None,
    line: None,
  };

  assert_eq!(detect_yaml_type(&plain("0755")), YerbaValueType::Integer);
  assert_eq!(detect_yaml_type(&plain("12:30:00")), YerbaValueType::Integer);
  assert_eq!(detect_yaml_type(&plain("1:30.5")), YerbaValueType::Float);
  assert_eq!(detect_yaml_type(&plain(".Inf")), YerbaValueType::Float);
  assert_eq!(detect_yaml_type(&plain("tRuE")), YerbaValueType::Boolean);
  assert_eq!(detect_yaml_type(&plain("08")), YerbaValueType::String);
  assert_eq!(detect_yaml_type(&plain("0o17")), YerbaValueType::String);
  assert_eq!(detect_yaml_type(&plain("1e5")), YerbaValueType::String);
}

#[test]
fn test_get_value_resolves_plain_scalars_like_psych() {
  let document = parse(indoc! {r#"
    mode: 0755
    time: 12:30:00
    size: 1_000
    legacy: 1,000
    flags: 0b1010
    duration: 1:30.5
    ratio: .5
    infinite: .Inf
    enabled: tRuE
    padded: 08
    quoted: "0755"
  "#});

  let number = |value: i64| yaml_serde::Value::Number(yaml_serde::Number::from(value));
  let decimal = |value: f64| yaml_serde::Value::Number(yaml_serde::Number::from(value));

  assert_eq!(document.get_value("mode"), Some(number(493)));
  assert_eq!(document.get_value("time"), Some(number(45000)));
  assert_eq!(document.get_value("size"), Some(number(1000)));
  assert_eq!(document.get_value("legacy"), Some(number(1000)));
  assert_eq!(document.get_value("flags"), Some(number(10)));
  assert_eq!(document.get_value("duration"), Some(decimal(5430.0)));
  assert_eq!(document.get_value("ratio"), Some(decimal(0.5)));
  assert_eq!(document.get_value("infinite"), Some(decimal(f64::INFINITY)));
  assert_eq!(document.get_value("enabled"), Some(yaml_serde::Value::Bool(true)));
  assert_eq!(document.get_value("padded"), Some(yaml_serde::Value::String("08".to_string())));
  assert_eq!(document.get_value("quoted"), Some(yaml_serde::Value::String("0755".to_string())));
}

#[test]
fn test_strings_that_read_as_numbers_are_quoted_when_written() {
  for value in ["0755", "0b1010", "1_000", "1,000", "12:30:00", "1:30.5", "1.", ".Inf", "tRuE", ".e+5"] {
    assert!(needs_quoting(value), "{value}");
  }
}

#[test]
fn test_yaml_1_2_numbers_are_still_quoted_when_written() {
  for value in ["08", "0o17", "1e5", "1.0e5", "+1E5"] {
    assert!(needs_quoting(value), "{value}");
  }
}

#[test]
fn test_set_quotes_strings_that_read_as_numbers() {
  let mut document = parse("mode: plain\n");

  document.set("mode", "0755").unwrap();

  assert_eq!(document.to_string(), "mode: \"0755\"\n");
  assert_eq!(document.get_value("mode"), Some(yaml_serde::Value::String("0755".to_string())));
}

#[test]
fn test_quote_style_leaves_plain_numbers_unquoted() {
  let mut document = parse(indoc! {"
    mode: 0755
    time: 12:30:00
    size: 1_000
    title: hello
  "});

  document.enforce_quotes(&yerba::QuoteStyle::Double).unwrap();

  assert_eq!(
    document.to_string(),
    indoc! {r#"
      mode: 0755
      time: 12:30:00
      size: 1_000
      title: "hello"
    "#}
  );
}
