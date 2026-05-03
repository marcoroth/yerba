//! FFI bindings for the Yerba C API.
//!
//! # Safety
//!
//! All functions in this module require that pointer arguments are either valid or null
//! (where documented as nullable). The caller is responsible for freeing returned pointers
//! using the corresponding `_free` functions.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use yaml_parser::SyntaxKind;

use crate::selector::Selector;
use crate::syntax::{detect_yaml_type, YerbaValueType};
use crate::{Document, InsertPosition, QuoteStyle};

#[repr(C)]
pub struct YerbaResult {
  pub success: bool,
  pub error: *mut c_char,
}

impl YerbaResult {
  fn ok() -> Self {
    YerbaResult {
      success: true,
      error: ptr::null_mut(),
    }
  }

  fn err(message: &str) -> Self {
    YerbaResult {
      success: false,
      error: CString::new(message).unwrap_or_default().into_raw(),
    }
  }
}

#[repr(C)]
pub struct YerbaTypedValue {
  pub text: *mut c_char,
  pub value_type: YerbaValueType,
}

#[repr(C)]
pub struct YerbaTypedList {
  pub json: *mut c_char,
  pub length: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YerbaNodeType {
  Scalar = 0,
  Map = 1,
  Sequence = 2,
  NotFound = 3,
}

#[repr(C)]
pub struct YerbaParseResult {
  pub document: *mut Document,
  pub error: *mut c_char,
}

#[repr(C)]
pub struct YerbaLocation {
  pub start_offset: usize,
  pub end_offset: usize,
  pub start_line: usize,
  pub start_column: usize,
  pub end_line: usize,
  pub end_column: usize,
}

#[repr(C)]
pub struct YerbaGetResult {
  pub is_list: bool,
  pub node_type: YerbaNodeType,
  pub single: YerbaTypedValue,
  pub list: YerbaTypedList,
  pub location: YerbaLocation,
  pub key_name: *mut c_char,
  pub key_location: YerbaLocation,
  pub error: *mut c_char,
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_parse_file(path: *const c_char) -> YerbaParseResult {
  if path.is_null() {
    return YerbaParseResult {
      document: ptr::null_mut(),
      error: CString::new("path is null").unwrap_or_default().into_raw(),
    };
  }

  let path_string = match CStr::from_ptr(path).to_str() {
    Ok(string) => string,
    Err(e) => {
      return YerbaParseResult {
        document: ptr::null_mut(),
        error: CString::new(format!("Invalid UTF-8 in path: {}", e))
          .unwrap_or_default()
          .into_raw(),
      }
    }
  };

  match Document::parse_file(path_string) {
    Ok(document) => YerbaParseResult {
      document: Box::into_raw(Box::new(document)),
      error: ptr::null_mut(),
    },

    Err(e) => YerbaParseResult {
      document: ptr::null_mut(),
      error: CString::new(e.to_string()).unwrap_or_default().into_raw(),
    },
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_parse(content: *const c_char) -> YerbaParseResult {
  if content.is_null() {
    return YerbaParseResult {
      document: ptr::null_mut(),
      error: CString::new("content is null").unwrap_or_default().into_raw(),
    };
  }

  let content_string = match CStr::from_ptr(content).to_str() {
    Ok(string) => string,
    Err(e) => {
      return YerbaParseResult {
        document: ptr::null_mut(),
        error: CString::new(format!("Invalid UTF-8: {}", e))
          .unwrap_or_default()
          .into_raw(),
      }
    }
  };

  match Document::parse(content_string) {
    Ok(document) => YerbaParseResult {
      document: Box::into_raw(Box::new(document)),
      error: ptr::null_mut(),
    },

    Err(e) => YerbaParseResult {
      document: ptr::null_mut(),
      error: CString::new(e.to_string()).unwrap_or_default().into_raw(),
    },
  }
}

fn compute_location(source: &str, start_offset: usize, end_offset: usize) -> YerbaLocation {
  let start = start_offset.min(source.len());
  let end = end_offset.min(source.len());

  let before_start = &source[..start];
  let start_line = before_start.chars().filter(|c| *c == '\n').count() + 1;
  let start_column = start - before_start.rfind('\n').map(|p| p + 1).unwrap_or(0);

  let before_end = &source[..end];
  let end_line = before_end.chars().filter(|c| *c == '\n').count() + 1;
  let end_column = end - before_end.rfind('\n').map(|p| p + 1).unwrap_or(0);

  YerbaLocation {
    start_offset: start,
    end_offset: end,
    start_line,
    start_column,
    end_line,
    end_column,
  }
}

const EMPTY_LOCATION: YerbaLocation = YerbaLocation {
  start_offset: 0,
  end_offset: 0,
  start_line: 0,
  start_column: 0,
  end_line: 0,
  end_column: 0,
};

#[no_mangle]
pub unsafe extern "C" fn yerba_document_free(document: *mut Document) {
  if !document.is_null() {
    drop(Box::from_raw(document));
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_get(document: *const Document, path: *const c_char) -> YerbaGetResult {
  let document = &*document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let selector = Selector::parse(path_string);

  if let Err(e) = Document::validate_path(path_string) {
    return YerbaGetResult {
      is_list: false,
      node_type: YerbaNodeType::NotFound,
      single: YerbaTypedValue {
        text: ptr::null_mut(),
        value_type: YerbaValueType::Null,
      },
      list: YerbaTypedList {
        json: ptr::null_mut(),
        length: 0,
      },
      key_name: ptr::null_mut(),
      key_location: EMPTY_LOCATION,
      location: EMPTY_LOCATION,
      error: CString::new(e.to_string()).unwrap_or_default().into_raw(),
    };
  }

  let source = document.to_string();

  let (location, key_text, key_location) = match document.navigate(path_string) {
    Ok(node) => {
      let range = node.text_range();
      let location = compute_location(&source, range.start().into(), range.end().into());

      let (key_text, key_location) = node
        .parent()
        .and_then(|parent| {
          use rowan::ast::AstNode;
          use yaml_parser::ast::BlockMapEntry;

          BlockMapEntry::cast(parent).and_then(|entry| {
            entry.key().and_then(|key_node| {
              let key_text = crate::syntax::extract_scalar_text(key_node.syntax())?;
              let key_range = key_node.syntax().text_range();
              let key_location = compute_location(&source, key_range.start().into(), key_range.end().into());

              Some((key_text, key_location))
            })
          })
        })
        .map(|(name, location)| (Some(name), location))
        .unwrap_or((None, EMPTY_LOCATION));

      (location, key_text, key_location)
    }
    Err(_) => (EMPTY_LOCATION, None, EMPTY_LOCATION),
  };

  let key_name_pointer = key_text
    .map(|name| CString::new(name).unwrap_or_default().into_raw())
    .unwrap_or(ptr::null_mut());

  if selector.has_wildcard() {
    let values = document.get_all_typed(path_string);

    let typed: Vec<serde_json::Value> = values
      .iter()
      .map(|serde_value| {
        serde_json::json!({
          "text": serde_value.text,
          "type": detect_yaml_type(serde_value) as u8
        })
      })
      .collect();

    let json = serde_json::to_string(&typed).unwrap_or_else(|_| "[]".to_string());
    let length = values.len();

    YerbaGetResult {
      is_list: true,
      node_type: YerbaNodeType::Sequence,
      single: YerbaTypedValue {
        text: ptr::null_mut(),
        value_type: YerbaValueType::Null,
      },
      list: YerbaTypedList {
        json: CString::new(json).unwrap_or_default().into_raw(),
        length,
      },
      location,
      key_name: key_name_pointer,
      key_location,
      error: ptr::null_mut(),
    }
  } else {
    match document.get_typed(path_string) {
      Some(scalar) => {
        let vtype = detect_yaml_type(&scalar);

        YerbaGetResult {
          is_list: false,
          node_type: YerbaNodeType::Scalar,
          single: YerbaTypedValue {
            text: CString::new(scalar.text).unwrap_or_default().into_raw(),
            value_type: vtype,
          },
          list: YerbaTypedList {
            json: ptr::null_mut(),
            length: 0,
          },
          location,
          key_name: key_name_pointer,
          key_location,
          error: ptr::null_mut(),
        }
      }

      None => {
        use rowan::ast::AstNode;
        use yaml_parser::ast::{BlockMap, BlockSeq};

        let node_type = match document.navigate(path_string) {
          Ok(node) => {
            if node.children().any(|child| BlockSeq::can_cast(child.kind())) {
              YerbaNodeType::Sequence
            } else if node.children().any(|child| BlockMap::can_cast(child.kind())) {
              YerbaNodeType::Map
            } else if node.descendants().any(|child| BlockSeq::can_cast(child.kind())) {
              YerbaNodeType::Sequence
            } else if node.descendants().any(|child| BlockMap::can_cast(child.kind())) {
              YerbaNodeType::Map
            } else {
              YerbaNodeType::NotFound
            }
          }
          Err(_) => YerbaNodeType::NotFound,
        };

        YerbaGetResult {
          is_list: false,
          node_type,
          single: YerbaTypedValue {
            text: ptr::null_mut(),
            value_type: YerbaValueType::Null,
          },
          list: YerbaTypedList {
            json: ptr::null_mut(),
            length: 0,
          },
          location,
          key_name: key_name_pointer,
          key_location,
          error: ptr::null_mut(),
        }
      }
    }
  }
}

/// Caller must free with yerba_string_free.
#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_value(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.get_value(path_string) {
    Some(value) => {
      let json = crate::json::yaml_to_json(&value);
      let json_string = serde_json::to_string(&json).unwrap_or_else(|_| "null".to_string());
      CString::new(json_string).unwrap_or_default().into_raw()
    }
    None => ptr::null_mut(),
  }
}

/// Caller must free with yerba_string_free.
#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_values(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let values = document.get_values(path_string);
  let json_values: Vec<serde_json::Value> = values.iter().map(crate::json::yaml_to_json).collect();
  let json_string = serde_json::to_string(&json_values).unwrap_or_else(|_| "[]".to_string());

  CString::new(json_string).unwrap_or_default().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_quote_style(document: *const Document, path: *const c_char) -> i32 {
  let document = &*document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.get_typed(path_string) {
    Some(scalar) => match scalar.kind {
      SyntaxKind::PLAIN_SCALAR => 0,
      SyntaxKind::SINGLE_QUOTED_SCALAR => 1,
      SyntaxKind::DOUBLE_QUOTED_SCALAR => 2,
      _ => -1,
    },

    None => -1,
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_set_quote_style(
  document: *mut Document,
  path: *const c_char,
  style: i32,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let quote_style = match style {
    0 => QuoteStyle::Plain,
    1 => QuoteStyle::Single,
    2 => QuoteStyle::Double,
    _ => return YerbaResult::err("Invalid quote style (use 0=plain, 1=single, 2=double)"),
  };

  match document.set_scalar_style(path_string, &quote_style) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_evaluate_condition(
  document: *const Document,
  parent_path: *const c_char,
  condition: *const c_char,
) -> bool {
  let document = &*document;
  let parent = CStr::from_ptr(parent_path).to_str().unwrap_or("");
  let cond = CStr::from_ptr(condition).to_str().unwrap_or("");
  document.evaluate_condition(parent, cond)
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_exists(document: *const Document, path: *const c_char) -> bool {
  let document = &*document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");
  document.exists(path_string)
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_find(
  document: *const Document,
  path: *const c_char,
  condition: *const c_char,
  select: *const c_char,
) -> *mut c_char {
  let document = &*document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let condition_str = if condition.is_null() {
    None
  } else {
    CStr::from_ptr(condition).to_str().ok()
  };

  let _select_string = if select.is_null() {
    None
  } else {
    CStr::from_ptr(select).to_str().ok()
  };

  let values = match condition_str {
    Some(cond) => document.filter(path_string, cond),
    None => document.get_values(path_string),
  };

  let select_fields: Option<Vec<&str>> = _select_string.map(|s| s.split(',').collect());

  let mut results: Vec<serde_json::Value> = Vec::new();

  for value in &values {
    match &select_fields {
      Some(fields) => {
        let mut result = serde_json::Map::new();

        for field in fields {
          let json_value = crate::json::resolve_select_field(value, field);
          let json_key = crate::json::select_field_key(field);

          result.insert(json_key, json_value);
        }

        results.push(serde_json::Value::Object(result));
      }
      None => {
        results.push(crate::json::yaml_to_json(value));
      }
    }
  }

  let json = serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string());
  CString::new(json).unwrap_or_default().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_set(
  document: *mut Document,
  path: *const c_char,
  value: *const c_char,
  value_type: YerbaValueType,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let value_string = CStr::from_ptr(value).to_str().unwrap_or("");

  let result = match value_type {
    YerbaValueType::String => document.set(path_string, value_string),
    _ => document.set_plain(path_string, value_string),
  };

  match result {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_insert(
  document: *mut Document,
  path: *const c_char,
  value: *const c_char,
  before: *const c_char,
  after: *const c_char,
  at: i64,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let value_string = CStr::from_ptr(value).to_str().unwrap_or("");

  let position = if at >= 0 {
    InsertPosition::At(at as usize)
  } else if !before.is_null() {
    let target = CStr::from_ptr(before).to_str().unwrap_or("");
    InsertPosition::Before(target.to_string())
  } else if !after.is_null() {
    let target = CStr::from_ptr(after).to_str().unwrap_or("");
    InsertPosition::After(target.to_string())
  } else {
    InsertPosition::Last
  };

  match document.insert_into(path_string, value_string, position) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_insert_object(
  document: *mut Document,
  path: *const c_char,
  json: *const c_char,
  before: *const c_char,
  after: *const c_char,
  at: i64,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let json_string = CStr::from_ptr(json).to_str().unwrap_or("");

  let json_value: serde_json::Value = match serde_json::from_str(json_string) {
    Ok(v) => v,
    Err(e) => return YerbaResult::err(&format!("Invalid JSON: {}", e)),
  };

  let try_paths = if path_string.is_empty() {
    vec!["[]".to_string(), "[0]".to_string()]
  } else {
    vec![format!("{}[]", path_string), format!("{}[0]", path_string)]
  };

  let mut quote_style = QuoteStyle::Plain;

  'outer: for try_path in &try_paths {
    for scalar in document.get_all_typed(try_path) {
      if scalar.kind == SyntaxKind::DOUBLE_QUOTED_SCALAR {
        quote_style = QuoteStyle::Double;
        break 'outer;
      } else if scalar.kind == SyntaxKind::SINGLE_QUOTED_SCALAR {
        quote_style = QuoteStyle::Single;
        break 'outer;
      }
    }
  }

  if quote_style == QuoteStyle::Plain {
    if let Some(serde_yaml::Value::Sequence(seq)) = document.get_value(path_string).as_ref() {
      if let Some(serde_yaml::Value::Mapping(map)) = seq.first() {
        if let Some((serde_yaml::Value::String(key_name), _)) = map.iter().next() {
          let deep_path = if path_string.is_empty() {
            format!("[].{}", key_name)
          } else {
            format!("{}[].{}", path_string, key_name)
          };

          for scalar in document.get_all_typed(&deep_path) {
            if scalar.kind == SyntaxKind::DOUBLE_QUOTED_SCALAR {
              quote_style = QuoteStyle::Double;
              break;
            } else if scalar.kind == SyntaxKind::SINGLE_QUOTED_SCALAR {
              quote_style = QuoteStyle::Single;
              break;
            }
          }
        }
      }
    }
  }

  let yaml_text = crate::yaml_writer::json_to_yaml_text(&json_value, &quote_style, 0);

  let position = if at >= 0 {
    InsertPosition::At(at as usize)
  } else if !before.is_null() {
    let target = CStr::from_ptr(before).to_str().unwrap_or("");
    InsertPosition::Before(target.to_string())
  } else if !after.is_null() {
    let target = CStr::from_ptr(after).to_str().unwrap_or("");
    InsertPosition::After(target.to_string())
  } else {
    InsertPosition::Last
  };

  match document.insert_into(path_string, &yaml_text, position) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_delete(document: *mut Document, path: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.delete(path_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_remove(
  document: *mut Document,
  path: *const c_char,
  value: *const c_char,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let value_string = CStr::from_ptr(value).to_str().unwrap_or("");

  match document.remove(path_string, value_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_remove_at(
  document: *mut Document,
  path: *const c_char,
  index: usize,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.remove_at(path_string, index) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_rename(
  document: *mut Document,
  source: *const c_char,
  dest: *const c_char,
) -> YerbaResult {
  let document = &mut *document;
  let source_string = CStr::from_ptr(source).to_str().unwrap_or("");
  let dest_string = CStr::from_ptr(dest).to_str().unwrap_or("");

  match document.rename(source_string, dest_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_sort(
  document: *mut Document,
  path: *const c_char,
  by: *const c_char,
  case_sensitive: bool,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let by_string = if by.is_null() {
    None
  } else {
    CStr::from_ptr(by).to_str().ok()
  };

  let sort_fields: Vec<crate::SortField> = match by_string {
    Some(fields) => fields
      .split(',')
      .map(|field| {
        if let Some(name) = field.strip_suffix(":desc") {
          crate::SortField {
            path: name.to_string(),
            ascending: false,
          }
        } else {
          crate::SortField {
            path: field.to_string(),
            ascending: true,
          }
        }
      })
      .collect(),
    None => vec![],
  };

  match document.sort_items(path_string, &sort_fields, case_sensitive) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_sort_keys(
  document: *mut Document,
  path: *const c_char,
  order: *const c_char,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let order_string = CStr::from_ptr(order).to_str().unwrap_or("");
  let key_order: Vec<&str> = order_string.split(',').collect();

  match document.sort_keys(path_string, &key_order) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_quote_style(
  document: *mut Document,
  path: *const c_char,
  key_style: *const c_char,
  value_style: *const c_char,
) -> YerbaResult {
  let document = &mut *document;

  let path_string = if path.is_null() {
    None
  } else {
    CStr::from_ptr(path).to_str().ok()
  };

  let key_quote_style = if key_style.is_null() {
    None
  } else {
    CStr::from_ptr(key_style)
      .to_str()
      .ok()
      .and_then(|s| s.parse::<QuoteStyle>().ok())
  };

  let value_quote_style = if value_style.is_null() {
    None
  } else {
    CStr::from_ptr(value_style)
      .to_str()
      .ok()
      .and_then(|s| s.parse::<QuoteStyle>().ok())
  };

  if let Some(ref key_style) = key_quote_style {
    if let Err(e) = document.enforce_key_style(key_style, path_string) {
      return YerbaResult::err(&e.to_string());
    }
  }

  if let Some(ref value_style) = value_quote_style {
    if let Err(e) = document.enforce_quotes_at(value_style, path_string) {
      return YerbaResult::err(&e.to_string());
    }
  }

  YerbaResult::ok()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_blank_lines(
  document: *mut Document,
  path: *const c_char,
  count: usize,
) -> YerbaResult {
  let document = &mut *document;
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.enforce_blank_lines(path_string, count) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_to_string(document: *const Document) -> *mut c_char {
  let document = &*document;
  let content = document.to_string();

  CString::new(content).unwrap_or_default().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_string_free(s: *mut c_char) {
  if !s.is_null() {
    drop(CString::from_raw(s));
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_result_free(result: YerbaResult) {
  if !result.error.is_null() {
    drop(CString::from_raw(result.error));
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_get_result_free(result: YerbaGetResult) {
  if !result.single.text.is_null() {
    drop(CString::from_raw(result.single.text));
  }

  if !result.list.json.is_null() {
    drop(CString::from_raw(result.list.json));
  }

  if !result.key_name.is_null() {
    drop(CString::from_raw(result.key_name));
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_glob_get(glob_pattern: *const c_char, path: *const c_char) -> YerbaTypedList {
  let pattern = CStr::from_ptr(glob_pattern).to_str().unwrap_or("");
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let selector = Selector::parse(path_string);

  let files = match glob::glob(pattern) {
    Ok(paths) => paths.filter_map(|p| p.ok()).collect::<Vec<_>>(),
    Err(_) => {
      return YerbaTypedList {
        json: CString::new("[]").unwrap_or_default().into_raw(),
        length: 0,
      }
    }
  };

  use rayon::prelude::*;

  let results: Vec<serde_json::Value> = files
    .par_iter()
    .flat_map(|file| {
      let mut file_results = Vec::new();

      if let Ok(document) = Document::parse_file(file) {
        if selector.has_wildcard() {
          for scalar in document.get_all_typed(path_string) {
            let value_type = detect_yaml_type(&scalar);

            file_results.push(serde_json::json!({"text": scalar.text, "type": value_type as u8}));
          }
        } else if let Some(scalar) = document.get_typed(path_string) {
          let value_type = detect_yaml_type(&scalar);

          file_results.push(serde_json::json!({"text": scalar.text, "type": value_type as u8}));
        }
      }

      file_results
    })
    .collect();

  let length = results.len();
  let json = serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string());

  YerbaTypedList {
    json: CString::new(json).unwrap_or_default().into_raw(),
    length,
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_glob_find(
  glob_pattern: *const c_char,
  path: *const c_char,
  condition: *const c_char,
  select: *const c_char,
) -> YerbaTypedList {
  let pattern = CStr::from_ptr(glob_pattern).to_str().unwrap_or("");
  let path_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let condition_string = if condition.is_null() {
    None
  } else {
    CStr::from_ptr(condition).to_str().ok()
  };

  let _select_string = if select.is_null() {
    None
  } else {
    CStr::from_ptr(select).to_str().ok()
  };

  let files = match glob::glob(pattern) {
    Ok(paths) => paths.filter_map(|p| p.ok()).collect::<Vec<_>>(),
    Err(_) => {
      return YerbaTypedList {
        json: CString::new("[]").unwrap_or_default().into_raw(),
        length: 0,
      }
    }
  };

  use rayon::prelude::*;

  let select_fields: Option<Vec<&str>> = _select_string.map(|s| s.split(',').collect());

  let all_results: Vec<serde_json::Value> = files
    .par_iter()
    .flat_map(|file| {
      let mut file_results = Vec::new();

      if let Ok(document) = Document::parse_file(file) {
        let values = match condition_string {
          Some(cond) => document.filter(path_string, cond),
          None => document.get_values(path_string),
        };

        let file_string = file.to_string_lossy().to_string();

        for value in &values {
          let mut result = serde_json::Map::new();
          result.insert("__file".to_string(), serde_json::Value::String(file_string.clone()));

          match &select_fields {
            Some(fields) => {
              for field in fields {
                let json_value = crate::json::resolve_select_field(value, field);
                let json_key = crate::json::select_field_key(field);
                result.insert(json_key, json_value);
              }
            }

            None => {
              if let serde_yaml::Value::Mapping(map) = value {
                for (key, yaml_value) in map {
                  let json_key = match key {
                    serde_yaml::Value::String(string) => string.clone(),
                    _ => format!("{:?}", key),
                  };

                  result.insert(json_key, crate::json::yaml_to_json(yaml_value));
                }
              }
            }
          }

          file_results.push(serde_json::Value::Object(result));
        }
      }

      file_results
    })
    .collect();

  let length = all_results.len();
  let json = serde_json::to_string_pretty(&all_results).unwrap_or_else(|_| "[]".to_string());

  YerbaTypedList {
    json: CString::new(json).unwrap_or_default().into_raw(),
    length,
  }
}
