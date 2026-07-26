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

use crate::syntax::{detect_yaml_type, YerbaValueType};
use crate::NodeType;
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
  pub node_type: NodeType,
  pub single: YerbaTypedValue,
  pub list: YerbaTypedList,
  pub location: YerbaLocation,
  pub key_name: *mut c_char,
  pub key_location: YerbaLocation,
  pub error: *mut c_char,
}

impl YerbaGetResult {
  fn empty() -> Self {
    YerbaGetResult {
      is_list: false,
      node_type: NodeType::NotFound,
      single: YerbaTypedValue {
        text: ptr::null_mut(),
        value_type: YerbaValueType::Null,
      },
      list: YerbaTypedList {
        json: ptr::null_mut(),
        length: 0,
      },
      location: EMPTY_LOCATION,
      key_name: ptr::null_mut(),
      key_location: EMPTY_LOCATION,
      error: ptr::null_mut(),
    }
  }

  fn with_error(error: &str) -> Self {
    let mut result = Self::empty();

    result.error = CString::new(error).unwrap_or_default().into_raw();

    result
  }

  fn with_location(mut self, location: YerbaLocation, key_name: *mut c_char, key_location: YerbaLocation) -> Self {
    self.location = location;
    self.key_name = key_name;
    self.key_location = key_location;

    self
  }

  fn scalar(mut self, text: &str, value_type: YerbaValueType) -> Self {
    self.node_type = NodeType::Scalar;

    self.single = YerbaTypedValue {
      text: CString::new(text).unwrap_or_default().into_raw(),
      value_type,
    };

    self
  }

  fn list(mut self, json: &str, length: usize) -> Self {
    self.is_list = true;
    self.node_type = NodeType::Sequence;

    self.list = YerbaTypedList {
      json: CString::new(json).unwrap_or_default().into_raw(),
      length,
    };

    self
  }

  fn with_node_type(mut self, node_type: NodeType) -> Self {
    self.node_type = node_type;

    self
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_parse_file(path: *const c_char) -> YerbaParseResult {
  if path.is_null() {
    return YerbaParseResult {
      document: ptr::null_mut(),
      error: CString::new("path is null").unwrap_or_default().into_raw(),
    };
  }

  let file_path = match CStr::from_ptr(path).to_str() {
    Ok(string) => string,
    Err(e) => {
      return YerbaParseResult {
        document: ptr::null_mut(),
        error: CString::new(format!("Invalid UTF-8 in path: {}", e)).unwrap_or_default().into_raw(),
      }
    }
  };

  match Document::parse_file(file_path) {
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
        error: CString::new(format!("Invalid UTF-8: {}", e)).unwrap_or_default().into_raw(),
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
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  if let Err(e) = Document::validate_path(selector_string) {
    return YerbaGetResult::with_error(&e.to_string());
  }

  let info = document.get_node_info(selector_string);
  let location = location_to_ffi(info.location);
  let key_location = location_to_ffi(info.key_location);

  let key_name = info
    .key_name
    .map(|name| CString::new(name).unwrap_or_default().into_raw())
    .unwrap_or(ptr::null_mut());

  let base = YerbaGetResult::empty().with_location(location, key_name, key_location);

  if info.is_list {
    let typed: Vec<serde_json::Value> = info
      .list_values
      .iter()
      .map(|value| {
        serde_json::json!({
          "text": value.text,
          "type": detect_yaml_type(value) as u8
        })
      })
      .collect();

    let json = serde_json::to_string(&typed).unwrap_or_else(|_| "[]".to_string());

    base.list(&json, info.list_values.len())
  } else {
    match info.value {
      Some(scalar) => base.scalar(&scalar.text, detect_yaml_type(&scalar)),
      None => base.with_node_type(info.node_type),
    }
  }
}

fn location_to_ffi(location: crate::Location) -> YerbaLocation {
  YerbaLocation {
    start_offset: location.start_offset,
    end_offset: location.end_offset,
    start_line: location.start_line,
    start_column: location.start_column,
    end_line: location.end_line,
    end_column: location.end_column,
  }
}

/// Caller must free with yerba_string_free.
#[no_mangle]
pub unsafe extern "C" fn yerba_document_source(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.source(selector_string) {
    Ok(text) => CString::new(text).unwrap_or_default().into_raw(),
    Err(_) => ptr::null_mut(),
  }
}

/// Caller must free with yerba_string_free.
#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_value(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.get_value(selector_string) {
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
pub unsafe extern "C" fn yerba_document_selectors(document: *const Document) -> *mut c_char {
  let document = &*document;

  let selectors = document.selectors();
  let json_string = serde_json::to_string(&selectors).unwrap_or_else(|_| "[]".to_string());

  CString::new(json_string).unwrap_or_default().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_resolve_selectors(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let selectors = document.resolve_selectors(selector_string);
  let json_string = serde_json::to_string(&selectors).unwrap_or_else(|_| "[]".to_string());

  CString::new(json_string).unwrap_or_default().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_selector_has_wildcard(path: *const c_char) -> bool {
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  crate::selector::Selector::parse(selector_string).has_wildcard()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_keys(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let keys = document.keys_at(selector_string);
  let json_string = serde_json::to_string(&keys).unwrap_or_else(|_| "[]".to_string());

  CString::new(json_string).unwrap_or_default().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_quote_style(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.get_quote_style(selector_string) {
    Some(style) => {
      let ruby_style = style.replace('-', "_");

      CString::new(ruby_style).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut())
    }

    None => std::ptr::null_mut(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_set_quote_style(document: *mut Document, path: *const c_char, style: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let style_string = &CStr::from_ptr(style).to_str().unwrap_or("").replace('_', "-");

  let quote_style = match style_string.parse::<QuoteStyle>() {
    Ok(style) => style,
    Err(e) => return YerbaResult::err(&e),
  };

  match document.enforce_quotes_at(&quote_style, Some(selector_string)) {
    Ok(_warnings) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_collection_style(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.get_collection_style(selector_string) {
    Some(style) => CString::new(style).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut()),
    None => std::ptr::null_mut(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_set_collection_style(document: *mut Document, path: *const c_char, style: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let style_string = CStr::from_ptr(style).to_str().unwrap_or("");

  match document.set_collection_style(selector_string, style_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_sequence_indent(document: *const Document, path: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.get_sequence_indent(selector_string) {
    Some(style) => CString::new(style).map(|s| s.into_raw()).unwrap_or(std::ptr::null_mut()),
    None => std::ptr::null_mut(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_set_sequence_indent(document: *mut Document, path: *const c_char, style: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let style_string = CStr::from_ptr(style).to_str().unwrap_or("");

  match document.set_sequence_indent(selector_string, style_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_validate_condition(condition: *const c_char, item_context: bool) -> *mut c_char {
  if condition.is_null() {
    return ptr::null_mut();
  }

  let cond = CStr::from_ptr(condition).to_str().unwrap_or("");

  let result = if item_context {
    crate::validate_item_condition(cond)
  } else {
    crate::validate_condition(cond)
  };

  match result {
    Ok(()) => ptr::null_mut(),
    Err(error) => CString::new(error.to_string()).unwrap_or_default().into_raw(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_evaluate_condition(document: *const Document, parent_path: *const c_char, condition: *const c_char) -> bool {
  let document = &*document;
  let parent = CStr::from_ptr(parent_path).to_str().unwrap_or("");
  let cond = CStr::from_ptr(condition).to_str().unwrap_or("");

  document.evaluate_condition(parent, cond).unwrap_or(false)
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_exists(document: *const Document, path: *const c_char) -> bool {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  document.exists(selector_string)
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_valid_selector(document: *const Document, path: *const c_char) -> bool {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  document.is_valid_selector(selector_string)
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_find(document: *const Document, path: *const c_char, condition: *const c_char, select: *const c_char) -> *mut c_char {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let condition_string = if condition.is_null() { None } else { CStr::from_ptr(condition).to_str().ok() };

  let select_string = if select.is_null() { None } else { CStr::from_ptr(select).to_str().ok() };

  let results = document.find_items(selector_string, condition_string, select_string).unwrap_or_default();
  let json = serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string());

  CString::new(json).unwrap_or_default().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_set(
  document: *mut Document,
  path: *const c_char,
  value: *const c_char,
  value_type: YerbaValueType,
  all: bool,
) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let value_string = CStr::from_ptr(value).to_str().unwrap_or("");

  let result = if all {
    document.set_all(selector_string, value_string)
  } else {
    match value_type {
      YerbaValueType::String => document.set(selector_string, value_string),
      _ => document.set_plain(selector_string, value_string),
    }
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
  value_type: YerbaValueType,
  before: *const c_char,
  after: *const c_char,
  at: i64,
) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let raw_value = CStr::from_ptr(value).to_str().unwrap_or("");

  let value_string = match value_type {
    YerbaValueType::String => crate::syntax::quote_if_needed(raw_value),
    _ => raw_value.to_string(),
  };

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

  match document.insert_into(selector_string, &value_string, position) {
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
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let json_string = CStr::from_ptr(json).to_str().unwrap_or("");

  let json_value: serde_json::Value = match serde_json::from_str(json_string) {
    Ok(value) => value,
    Err(e) => return YerbaResult::err(&format!("Invalid JSON: {}", e)),
  };

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

  match document.insert_object(selector_string, &json_value, position) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_delete(document: *mut Document, path: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.delete(selector_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_insert_objects(document: *mut Document, path: *const c_char, json: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let json_string = CStr::from_ptr(json).to_str().unwrap_or("");

  let json_values: Vec<serde_json::Value> = match serde_json::from_str(json_string) {
    Ok(values) => values,
    Err(e) => return YerbaResult::err(&format!("Invalid JSON: {}", e)),
  };

  match document.insert_objects(selector_string, &json_values) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_remove(document: *mut Document, path: *const c_char, value: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let value_string = CStr::from_ptr(value).to_str().unwrap_or("");

  match document.remove(selector_string, value_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_remove_at(document: *mut Document, path: *const c_char, index: usize) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.remove_at(selector_string, index) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_move_item(document: *mut Document, path: *const c_char, from: usize, to: usize) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.move_item(selector_string, from, to) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_rename(document: *mut Document, source: *const c_char, dest: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let source_string = CStr::from_ptr(source).to_str().unwrap_or("");
  let dest_string = CStr::from_ptr(dest).to_str().unwrap_or("");

  match document.rename(source_string, dest_string) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_sort(document: *mut Document, path: *const c_char, by: *const c_char, case_sensitive: bool) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  let by_string = if by.is_null() { None } else { CStr::from_ptr(by).to_str().ok() };

  let sort_fields: Vec<crate::SortField> = match by_string {
    Some(fields) => crate::SortField::parse_list(fields),
    None => vec![],
  };

  match document.sort_items(selector_string, &sort_fields, case_sensitive) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_reorder(document: *mut Document, path: *const c_char, by: *const c_char, order_csv: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let by_string = CStr::from_ptr(by).to_str().unwrap_or("");
  let order_string = CStr::from_ptr(order_csv).to_str().unwrap_or("");

  let desired_order: Vec<&str> = order_string.split(',').map(|s| s.trim()).collect();

  match document.reorder_items(selector_string, by_string, &desired_order) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_sort_keys(document: *mut Document, path: *const c_char, order: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let order_string = CStr::from_ptr(order).to_str().unwrap_or("");
  let key_order: Vec<&str> = order_string.split(',').collect();

  match document.sort_keys(selector_string, &key_order) {
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

  let selector = if path.is_null() { None } else { CStr::from_ptr(path).to_str().ok() };

  let key = if key_style.is_null() {
    None
  } else {
    CStr::from_ptr(key_style).to_str().ok().and_then(|s| s.parse::<crate::KeyStyle>().ok())
  };

  let value = if value_style.is_null() {
    None
  } else {
    CStr::from_ptr(value_style).to_str().ok().and_then(|s| s.parse::<QuoteStyle>().ok())
  };

  match document.enforce_quote_style(key.as_ref(), value.as_ref(), selector) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_blank_lines(document: *mut Document, path: *const c_char, count: usize) -> YerbaResult {
  let document = &mut *document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  match document.enforce_blank_lines(selector_string, count) {
    Ok(()) => YerbaResult::ok(),
    Err(e) => YerbaResult::err(&e.to_string()),
  }
}

/// Caller must free with yerba_string_free.
#[no_mangle]
pub unsafe extern "C" fn yerba_document_validate_schema(document: *const Document, schema_json: *const c_char, selector: *const c_char) -> *mut c_char {
  let document = &*document;
  let schema_string = CStr::from_ptr(schema_json).to_str().unwrap_or("");
  let selector_string = if selector.is_null() { None } else { CStr::from_ptr(selector).to_str().ok() };

  let schema: serde_json::Value = match serde_json::from_str(schema_string) {
    Ok(schema) => schema,
    Err(e) => {
      let error = serde_json::json!([{"message": format!("invalid schema: {}", e), "path": "", "line": null}]);
      return CString::new(error.to_string()).unwrap_or_default().into_raw();
    }
  };

  let errors = document.validate_schema(&schema, false, selector_string);

  if errors.is_empty() {
    return ptr::null_mut();
  }

  let json_errors: Vec<serde_json::Value> = errors
    .iter()
    .map(|error| {
      serde_json::json!({
        "message": error.message,
        "path": error.path,
        "line": error.line,
        "item_label": error.item_label,
      })
    })
    .collect();

  CString::new(serde_json::to_string(&json_errors).unwrap_or_else(|_| "[]".to_string()))
    .unwrap_or_default()
    .into_raw()
}

/// Caller must free with yerba_string_free.
#[no_mangle]
pub unsafe extern "C" fn yerba_yerbafile_find(directory: *const c_char) -> *mut c_char {
  let start = if directory.is_null() {
    std::env::current_dir().ok()
  } else {
    CStr::from_ptr(directory).to_str().ok().map(std::path::PathBuf::from)
  };

  let path = match start {
    Some(dir) => crate::Yerbafile::find_from(dir),
    None => None,
  };

  match path {
    Some(path) => CString::new(path.to_string_lossy().to_string()).unwrap_or_default().into_raw(),
    None => ptr::null_mut(),
  }
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_apply_yerbafile(document: *mut Document, file_path: *const c_char, yerbafile_path: *const c_char) -> YerbaResult {
  let document = &mut *document;
  let file_path_string = CStr::from_ptr(file_path).to_str().unwrap_or("");

  let yerbafile = if yerbafile_path.is_null() {
    match crate::Yerbafile::find() {
      Some(path) => match crate::Yerbafile::load(&path) {
        Ok(yerbafile) => yerbafile,
        Err(e) => return YerbaResult::err(&e.to_string()),
      },

      None => return YerbaResult::err("No Yerbafile found"),
    }
  } else {
    let path = CStr::from_ptr(yerbafile_path).to_str().unwrap_or("");

    match crate::Yerbafile::load(path) {
      Ok(yerbafile) => yerbafile,
      Err(e) => return YerbaResult::err(&e.to_string()),
    }
  };

  match yerbafile.apply_to_document(document, file_path_string) {
    Ok(_) => YerbaResult::ok(),
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

fn located_nodes_to_list(nodes: &[crate::LocatedNode]) -> YerbaTypedList {
  let results: Vec<serde_json::Value> = nodes
    .iter()
    .map(|node| {
      let mut value = serde_json::json!({
        "node_type": node.node_type,
        "selector": node.selector,
        "line": node.line,
        "location": {
          "start_line": node.location.start_line,
          "start_column": node.location.start_column,
          "end_line": node.location.end_line,
          "end_column": node.location.end_column,
          "start_offset": node.location.start_offset,
          "end_offset": node.location.end_offset,
        },
      });

      if let Some(file_path) = &node.file_path {
        value["file_path"] = serde_json::json!(file_path);
      }

      if let Some(text) = &node.text {
        value["text"] = serde_json::json!(text);
      }

      if let Some(vt) = node.value_type {
        value["type"] = serde_json::json!(vt as u8);
      }

      if let Some(key_name) = &node.key_name {
        value["key_name"] = serde_json::json!(key_name);
        value["key_location"] = serde_json::json!({
          "start_line": node.key_location.start_line,
          "start_column": node.key_location.start_column,
          "end_line": node.key_location.end_line,
          "end_column": node.key_location.end_column,
          "start_offset": node.key_location.start_offset,
          "end_offset": node.key_location.end_offset,
        });
      }

      value
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
pub unsafe extern "C" fn yerba_glob_get(glob_pattern: *const c_char, path: *const c_char) -> YerbaTypedList {
  let pattern = CStr::from_ptr(glob_pattern).to_str().unwrap_or("");
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  located_nodes_to_list(&crate::glob_get(pattern, selector_string))
}

#[no_mangle]
pub unsafe extern "C" fn yerba_document_get_all(document: *const Document, path: *const c_char) -> YerbaTypedList {
  let document = &*document;
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");

  located_nodes_to_list(&document.get_all_located(selector_string))
}

#[no_mangle]
pub unsafe extern "C" fn yerba_glob_find(glob_pattern: *const c_char, path: *const c_char, condition: *const c_char, select: *const c_char) -> YerbaTypedList {
  let pattern = CStr::from_ptr(glob_pattern).to_str().unwrap_or("");
  let selector_string = CStr::from_ptr(path).to_str().unwrap_or("");
  let condition_string = if condition.is_null() { None } else { CStr::from_ptr(condition).to_str().ok() };
  let select_string = if select.is_null() { None } else { CStr::from_ptr(select).to_str().ok() };

  let all_results = crate::glob_find(pattern, selector_string, condition_string, select_string).unwrap_or_default();
  let length = all_results.len();
  let json = serde_json::to_string_pretty(&all_results).unwrap_or_else(|_| "[]".to_string());

  YerbaTypedList {
    json: CString::new(json).unwrap_or_default().into_raw(),
    length,
  }
}
