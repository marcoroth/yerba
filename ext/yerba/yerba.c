#include <ruby.h>
#include <ruby/encoding.h>
#include <string.h>
#include "include/yerba.h"

static VALUE rb_mYerba;
static VALUE rb_cDocument;
static VALUE rb_eError;
static VALUE rb_ePathNotFoundError;
static VALUE rb_eParseError;
static VALUE rb_ePathValidationError;

static void document_dfree(void *pointer) {
  if (pointer) yerba_document_free(pointer);
}

static size_t document_dsize(const void *pointer) {
  (void) pointer;

  return sizeof(void *);
}

static const rb_data_type_t document_type = {
  .wrap_struct_name = "Yerba::Document",
  .function = {
    .dfree = document_dfree,
    .dsize = document_dsize,
  },
  .flags = RUBY_TYPED_FREE_IMMEDIATELY,
};

static VALUE document_alloc(VALUE klass) {
  return TypedData_Wrap_Struct(klass, &document_type, NULL);
}

static inline struct Document *get_document(VALUE self) {
  struct Document *document;
  TypedData_Get_Struct(self, struct Document, &document_type, document);
  return document;
}

static VALUE make_utf8_string(const char *cstring) {
  return rb_enc_str_new_cstr(cstring, rb_utf8_encoding());
}

static void check_result(YerbaResult result) {
  if (!result.success) {
    VALUE message = make_utf8_string(result.error);
    VALUE error_class = rb_eError;

    if (strstr(result.error, "invalid path")) {
      error_class = rb_ePathValidationError;
    } else if (strstr(result.error, "path not found") || strstr(result.error, "not a sequence")) {
      error_class = rb_ePathNotFoundError;
    } else if (strstr(result.error, "parse error")) {
      error_class = rb_eParseError;
    }

    yerba_result_free(result);

    rb_raise(error_class, "%s", StringValueCStr(message));
  }
}

static VALUE typed_value_to_ruby(YerbaTypedValue typed_value) {
  if (typed_value.text == NULL) return Qnil;

  VALUE result;
  switch (typed_value.value_type) {
    case YERBA_VALUE_TYPE_NULL:
      result = Qnil;
      break;

    case YERBA_VALUE_TYPE_BOOLEAN:
      result = (strcmp(typed_value.text, "true") == 0 || strcmp(typed_value.text, "True") == 0 ||
                strcmp(typed_value.text, "TRUE") == 0 || strcmp(typed_value.text, "yes") == 0 ||
                strcmp(typed_value.text, "Yes") == 0 || strcmp(typed_value.text, "YES") == 0 ||
                strcmp(typed_value.text, "on") == 0 || strcmp(typed_value.text, "On") == 0 ||
                strcmp(typed_value.text, "ON") == 0 || strcmp(typed_value.text, "y") == 0 ||
                strcmp(typed_value.text, "Y") == 0)
               ? Qtrue : Qfalse;
      break;

    case YERBA_VALUE_TYPE_INTEGER:
      result = rb_cstr_to_inum(typed_value.text, 0, 0);
      break;

    case YERBA_VALUE_TYPE_FLOAT:
      result = rb_float_new(rb_cstr_to_dbl(typed_value.text, 0));
      break;

    case YERBA_VALUE_TYPE_STRING:
    default:
      result = make_utf8_string(typed_value.text);
      break;
  }

  return result;
}

static void check_condition(const char *condition, bool item_context) {
  if (!condition) return;

  char *error = yerba_validate_condition(condition, item_context);

  if (!error) return;

  VALUE message = make_utf8_string(error);
  yerba_string_free(error);

  rb_raise(rb_eError, "%s", StringValueCStr(message));
}

static int should_proceed(struct Document *document, VALUE opts) {
  if (NIL_P(opts)) return 1;
  VALUE v_condition = rb_hash_aref(opts, ID2SYM(rb_intern("condition")));
  if (NIL_P(v_condition)) return 1;

  VALUE v_path = rb_hash_aref(opts, ID2SYM(rb_intern("condition_path")));
  const char *parent_path = NIL_P(v_path) ? "" : StringValueCStr(v_path);

  check_condition(StringValueCStr(v_condition), parent_path[0] != '\0');

  return yerba_document_evaluate_condition(document, parent_path, StringValueCStr(v_condition));
}

/* Document.new(path) */
static VALUE document_initialize(int argc, VALUE *argv, VALUE self) {
  VALUE path;
  rb_scan_args(argc, argv, "01", &path);

  YerbaParseResult result;

  if (NIL_P(path)) {
    result = yerba_document_parse("---\n");
    rb_iv_set(self, "@path", Qnil);
    rb_iv_set(self, "@loaded_mtime", Qnil);
  } else {
    const char *file_path = StringValueCStr(path);
    result = yerba_document_parse_file(file_path);
    rb_iv_set(self, "@path", path);
    rb_iv_set(self, "@loaded_mtime", rb_funcall(rb_cFile, rb_intern("mtime"), 1, path));
  }

  if (!result.document) {
    VALUE message = make_utf8_string(result.error);
    yerba_string_free(result.error);

    rb_raise(rb_eParseError, "%s", StringValueCStr(message));
  }

  RTYPEDDATA_DATA(self) = result.document;

  return self;
}

/* document.replace_content!(content) — re-parse from YAML string, keeping the same Ruby object */
static VALUE document_replace_content(VALUE self, VALUE content) {
  const char *yaml_content = StringValueCStr(content);
  YerbaParseResult result = yerba_document_parse(yaml_content);

  if (!result.document) {
    VALUE message = make_utf8_string(result.error);
    yerba_string_free(result.error);

    rb_raise(rb_eParseError, "%s", StringValueCStr(message));
  }

  struct Document *old_document = get_document(self);

  if (old_document) {
    yerba_document_free(old_document);
  }

  RTYPEDDATA_DATA(self) = result.document;

  return self;
}

/* Document.parse(content) */
static VALUE document_s_parse(VALUE klass, VALUE content) {
  const char *yaml_content = StringValueCStr(content);
  YerbaParseResult result = yerba_document_parse(yaml_content);

  if (!result.document) {
    VALUE message = make_utf8_string(result.error);
    yerba_string_free(result.error);

    rb_raise(rb_eParseError, "%s", StringValueCStr(message));
  }

  VALUE instance = document_alloc(klass);
  RTYPEDDATA_DATA(instance) = result.document;
  rb_iv_set(instance, "@path", Qnil);

  return instance;
}

static VALUE location_to_ruby(YerbaLocation location) {
  VALUE klass = rb_path2class("Yerba::Location");

  return rb_funcall(klass, rb_intern("new"), 6,
    SIZET2NUM(location.start_line),
    SIZET2NUM(location.start_column),
    SIZET2NUM(location.end_line),
    SIZET2NUM(location.end_column),
    SIZET2NUM(location.start_offset),
    SIZET2NUM(location.end_offset)
  );
}

/* Build a Yerba::Scalar, Map, Sequence, or nil from a YerbaGetResult.
   Frees the result internally. */
static VALUE node_from_get_result(YerbaGetResult result, VALUE self, VALUE path) {
  if (result.error) {
    VALUE message = make_utf8_string(result.error);
    yerba_get_result_free(result);

    rb_raise(rb_ePathValidationError, "%s", StringValueCStr(message));
  }

  VALUE location = location_to_ruby(result.location);
  VALUE key = Qnil;

  if (result.key_name) {
    VALUE key_location = location_to_ruby(result.key_location);
    VALUE key_value = make_utf8_string(result.key_name);

    VALUE key_kwargs = rb_hash_new();
    rb_hash_aset(key_kwargs, ID2SYM(rb_intern("value")), key_value);
    key = rb_funcallv_kw(rb_path2class("Yerba::Scalar"), rb_intern("from_document"), 5, (VALUE[]){ Qnil, Qnil, key_location, Qnil, key_kwargs }, RB_PASS_KEYWORDS);
  }

  switch (result.node_type) {
    case NODE_TYPE_SCALAR: {
      VALUE value = typed_value_to_ruby(result.single);
      yerba_get_result_free(result);

      VALUE kwargs = rb_hash_new();
      rb_hash_aset(kwargs, ID2SYM(rb_intern("value")), value);

      return rb_funcallv_kw(rb_path2class("Yerba::Scalar"), rb_intern("from_document"), 5, (VALUE[]){ self, path, location, key, kwargs }, RB_PASS_KEYWORDS);
    }

    case NODE_TYPE_MAP:
      yerba_get_result_free(result);

      return rb_funcall(rb_path2class("Yerba::Map"), rb_intern("from_document"), 4, self, path, location, key);

    case NODE_TYPE_SEQUENCE:
      yerba_get_result_free(result);

      return rb_funcall(rb_path2class("Yerba::Sequence"), rb_intern("from_document"), 4, self, path, location, key);

    default:
      yerba_get_result_free(result);

      return Qnil;
  }
}

/* document[](path) → Yerba::Scalar, Yerba::Map, Yerba::Sequence, Array, or nil */
static VALUE document_bracket(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char index_buffer[32];

  if (RB_TYPE_P(path, T_FIXNUM)) {
    snprintf(index_buffer, sizeof(index_buffer), "[%ld]", FIX2LONG(path));
    path = rb_str_new_cstr(index_buffer);
  }

  const char *selector = StringValueCStr(path);

  if (yerba_selector_has_wildcard(selector)) {
    char *json = yerba_document_resolve_selectors(document, selector);

    if (!json) return rb_ary_new();

    VALUE json_string = make_utf8_string(json);
    yerba_string_free(json);

    VALUE resolved = rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
    long length = RARRAY_LEN(resolved);
    VALUE array = rb_ary_new_capa(length);

    for (long i = 0; i < length; i++) {
      VALUE concrete_path = rb_ary_entry(resolved, i);
      YerbaGetResult result = yerba_document_get(document, StringValueCStr(concrete_path));
      VALUE node = node_from_get_result(result, self, concrete_path);

      if (!NIL_P(node)) rb_ary_push(array, node);
    }

    return array;
  }

  YerbaGetResult result = yerba_document_get(document, selector);

  return node_from_get_result(result, self, path);
}

/* document.exists?(path) */
static VALUE document_exists_p(VALUE self, VALUE path) {
  struct Document *document = get_document(self);

  return yerba_document_exists(document, StringValueCStr(path)) ? Qtrue : Qfalse;
}

/* document.valid_selector?(selector) → true/false */
static VALUE document_valid_selector_p(VALUE self, VALUE path) {
  struct Document *document = get_document(self);

  return yerba_document_valid_selector(document, StringValueCStr(path)) ? Qtrue : Qfalse;
}

/* document.source(path) → raw YAML text of the node at path */
static VALUE document_source(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *text = yerba_document_source(document, StringValueCStr(path));

  if (!text) return Qnil;

  VALUE result = make_utf8_string(text);
  yerba_string_free(text);

  return result;
}

/* document.get_value(path) → parsed Ruby object (Hash/Array/String/Integer/etc) */
static VALUE document_get_value(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *json = yerba_document_get_value(document, StringValueCStr(path));

  if (!json) return Qnil;

  VALUE json_string = make_utf8_string(json);
  yerba_string_free(json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
}

/* document.selectors → ["database", "database.host", "tags", "tags[]", ...] */
static VALUE document_selectors(VALUE self) {
  struct Document *document = get_document(self);
  char *json = yerba_document_selectors(document);

  if (!json) return rb_ary_new();

  VALUE json_string = make_utf8_string(json);
  yerba_string_free(json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
}

/* document.keys_at(selector) → ["host", "port", ...] */
static VALUE document_keys_at(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *json = yerba_document_keys(document, StringValueCStr(path));

  if (!json) return rb_ary_new();

  VALUE json_string = make_utf8_string(json);
  yerba_string_free(json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
}

/* document.locations(selector) → [Yerba::Location, ...] */
static VALUE document_locations(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *json = yerba_document_resolve_selectors(document, StringValueCStr(path));

  if (!json) return rb_ary_new();

  VALUE json_string = make_utf8_string(json);
  yerba_string_free(json);

  VALUE resolved = rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
  long length = RARRAY_LEN(resolved);
  VALUE array = rb_ary_new_capa(length);

  for (long i = 0; i < length; i++) {
    VALUE concrete_path = rb_ary_entry(resolved, i);
    YerbaGetResult result = yerba_document_get(document, StringValueCStr(concrete_path));

    if (result.error) {
      yerba_get_result_free(result);
      continue;
    }

    if (result.location.start_line == 0 && result.location.end_line == 0) {
      yerba_get_result_free(result);
      continue;
    }

    VALUE location = location_to_ruby(result.location);
    yerba_get_result_free(result);
    rb_ary_push(array, location);
  }

  return array;
}

/* document.resolve_selectors(path) → ["[0].speakers[0]", "[0].speakers[1]", ...] */
static VALUE document_resolve_selectors(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *json = yerba_document_resolve_selectors(document, StringValueCStr(path));

  if (!json) return rb_ary_new();

  VALUE json_string = make_utf8_string(json);
  yerba_string_free(json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
}

/* document.get_quote_style(path) → :plain, :single, :double, :literal, etc. or nil */
static VALUE document_get_quote_style(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *style = yerba_document_get_quote_style(document, StringValueCStr(path));

  if (style == NULL) return Qnil;

  VALUE symbol = ID2SYM(rb_intern(style));

  yerba_string_free(style);

  return symbol;
}

/* document.set_quote_style(path, style) */
static VALUE document_set_quote_style(VALUE self, VALUE path, VALUE style) {
  struct Document *document = get_document(self);

  const char *style_string;

  if (RB_TYPE_P(style, T_SYMBOL)) {
    style_string = rb_id2name(SYM2ID(style));
  } else if (RB_TYPE_P(style, T_STRING)) {
    style_string = StringValueCStr(style);
  } else {
    rb_raise(rb_eError, "Invalid quote style (expected Symbol or String)");
  }

  YerbaResult result = yerba_document_set_quote_style(document, StringValueCStr(path), style_string);

  check_result(result);

  return self;
}

/* document.get_collection_style(path) → :flow, :block, or nil */
static VALUE document_get_collection_style(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *style = yerba_document_get_collection_style(document, StringValueCStr(path));

  if (style == NULL) return Qnil;

  VALUE symbol = ID2SYM(rb_intern(style));

  yerba_string_free(style);

  return symbol;
}

/* document.set_collection_style(path, style) */
static VALUE document_set_collection_style(VALUE self, VALUE path, VALUE style) {
  struct Document *document = get_document(self);

  const char *style_string;

  if (RB_TYPE_P(style, T_SYMBOL)) {
    style_string = rb_id2name(SYM2ID(style));
  } else if (RB_TYPE_P(style, T_STRING)) {
    style_string = StringValueCStr(style);
  } else {
    rb_raise(rb_eError, "Invalid collection style (expected Symbol or String)");
  }

  YerbaResult result = yerba_document_set_collection_style(document, StringValueCStr(path), style_string);

  check_result(result);

  return self;
}

/* document.get_sequence_indent(path) → :compact, :indented, or nil */
static VALUE document_get_sequence_indent(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *style = yerba_document_get_sequence_indent(document, StringValueCStr(path));

  if (style == NULL) return Qnil;

  VALUE symbol = ID2SYM(rb_intern(style));

  yerba_string_free(style);

  return symbol;
}

/* document.set_sequence_indent(path, style) */
static VALUE document_set_sequence_indent(VALUE self, VALUE path, VALUE style) {
  struct Document *document = get_document(self);

  const char *style_string;

  if (RB_TYPE_P(style, T_SYMBOL)) {
    style_string = rb_id2name(SYM2ID(style));
  } else if (RB_TYPE_P(style, T_STRING)) {
    style_string = StringValueCStr(style);
  } else {
    rb_raise(rb_eError, "Invalid sequence indent style (expected Symbol or String)");
  }

  YerbaResult result = yerba_document_set_sequence_indent(document, StringValueCStr(path), style_string);

  check_result(result);

  return self;
}

/* document.condition?(condition, path: "") */
static VALUE document_condition_p(int argc, VALUE *argv, VALUE self) {
  VALUE condition, opts;
  rb_scan_args(argc, argv, "1:", &condition, &opts);

  const char *parent_path = "";
  if (!NIL_P(opts)) {
    VALUE v_path = rb_hash_aref(opts, ID2SYM(rb_intern("path")));

    if (!NIL_P(v_path)) {
      parent_path = StringValueCStr(v_path);
    }
  }

  struct Document *document = get_document(self);

  check_condition(StringValueCStr(condition), parent_path[0] != '\0');

  return yerba_document_evaluate_condition(document, parent_path, StringValueCStr(condition)) ? Qtrue : Qfalse;
}

/* document.find(path, condition: nil, select: nil) */
static VALUE document_find(int argc, VALUE *argv, VALUE self) {
  VALUE path, opts;
  rb_scan_args(argc, argv, "1:", &path, &opts);

  const char *condition = NULL;
  const char *select = NULL;

  if (!NIL_P(opts)) {
    VALUE v_condition = rb_hash_aref(opts, ID2SYM(rb_intern("condition")));
    VALUE v_select = rb_hash_aref(opts, ID2SYM(rb_intern("select")));

    if (!NIL_P(v_condition)) condition = StringValueCStr(v_condition);
    if (!NIL_P(v_select)) select = StringValueCStr(v_select);
  }

  struct Document *document = get_document(self);

  check_condition(condition, true);

  char *json = yerba_document_find(document, StringValueCStr(path), condition, select);

  if (!json) return rb_ary_new();

  VALUE json_string = make_utf8_string(json);
  yerba_string_free(json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
}

/* Convert a Ruby VALUE to a C string + YerbaValueType pair.
   The caller must provide a number_buffer of at least 64 bytes. */
struct TypedValue {
  const char *text;
  YerbaValueType type;
};

static struct TypedValue ruby_to_typed_value(VALUE value, char *number_buffer) {
  struct TypedValue result;

  if (value == Qnil) {
    result.text = "null";
    result.type = YERBA_VALUE_TYPE_NULL;
  } else if (value == Qtrue) {
    result.text = "true";
    result.type = YERBA_VALUE_TYPE_BOOLEAN;
  } else if (value == Qfalse) {
    result.text = "false";
    result.type = YERBA_VALUE_TYPE_BOOLEAN;
  } else if (RB_INTEGER_TYPE_P(value)) {
    snprintf(number_buffer, 64, "%ld", NUM2LONG(value));
    result.text = number_buffer;
    result.type = YERBA_VALUE_TYPE_INTEGER;
  } else if (RB_FLOAT_TYPE_P(value)) {
    snprintf(number_buffer, 64, "%g", NUM2DBL(value));
    result.text = number_buffer;
    result.type = YERBA_VALUE_TYPE_FLOAT;
  } else {
    result.text = StringValueCStr(value);
    result.type = YERBA_VALUE_TYPE_STRING;
  }

  return result;
}

/* document.set(path, value, condition: nil, if_exists: false, if_missing: false) */
static VALUE document_set(int argc, VALUE *argv, VALUE self) {
  VALUE path, value, opts;
  rb_scan_args(argc, argv, "2:", &path, &value, &opts);

  struct Document *document = get_document(self);

  if (!should_proceed(document, opts)) return self;

  if (!NIL_P(opts)) {
    VALUE v_if_exists = rb_hash_aref(opts, ID2SYM(rb_intern("if_exists")));
    VALUE v_if_missing = rb_hash_aref(opts, ID2SYM(rb_intern("if_missing")));

    if (RTEST(v_if_exists) && !yerba_document_exists(document, StringValueCStr(path))) return self;
    if (RTEST(v_if_missing) && yerba_document_exists(document, StringValueCStr(path))) return self;
  }

  char number_buffer[64];
  struct TypedValue typed_value = ruby_to_typed_value(value, number_buffer);
  bool all = false;

  if (!NIL_P(opts)) {
    VALUE v_all = rb_hash_aref(opts, ID2SYM(rb_intern("all")));

    if (RTEST(v_all)) all = true;
  }

  YerbaResult result = yerba_document_set(document, StringValueCStr(path), typed_value.text, typed_value.type, all);
  check_result(result);

  return self;
}

/* document.insert(path, value, before: nil, after: nil, at: nil) */
static VALUE document_insert(int argc, VALUE *argv, VALUE self) {
  VALUE path, value, opts;
  rb_scan_args(argc, argv, "2:", &path, &value, &opts);

  const char *before = NULL;
  const char *after = NULL;
  long long at = -1;

  if (!NIL_P(opts)) {
    VALUE v_before = rb_hash_aref(opts, ID2SYM(rb_intern("before")));
    VALUE v_after = rb_hash_aref(opts, ID2SYM(rb_intern("after")));
    VALUE v_at = rb_hash_aref(opts, ID2SYM(rb_intern("at")));

    if (!NIL_P(v_before)) before = StringValueCStr(v_before);
    if (!NIL_P(v_after)) after = StringValueCStr(v_after);
    if (!NIL_P(v_at)) at = NUM2LL(v_at);
  }

  char number_buffer[64];
  struct TypedValue typed_value = ruby_to_typed_value(value, number_buffer);

  struct Document *document = get_document(self);
  YerbaResult result = yerba_document_insert(document, StringValueCStr(path), typed_value.text, typed_value.type, before, after, at);
  check_result(result);

  return self;
}

/* document.insert_object(path, object, before: nil, after: nil, at: nil) */
static VALUE document_insert_object(int argc, VALUE *argv, VALUE self) {
  VALUE path, object, opts;
  rb_scan_args(argc, argv, "2:", &path, &object, &opts);

  const char *before = NULL;
  const char *after = NULL;
  long long at = -1;

  if (!NIL_P(opts)) {
    VALUE v_before = rb_hash_aref(opts, ID2SYM(rb_intern("before")));
    VALUE v_after = rb_hash_aref(opts, ID2SYM(rb_intern("after")));
    VALUE v_at = rb_hash_aref(opts, ID2SYM(rb_intern("at")));

    if (!NIL_P(v_before)) before = StringValueCStr(v_before);
    if (!NIL_P(v_after)) after = StringValueCStr(v_after);
    if (!NIL_P(v_at)) at = NUM2LL(v_at);
  }

  struct Document *document = get_document(self);
  VALUE json_string = rb_funcall(rb_path2class("JSON"), rb_intern("generate"), 1, object);

  YerbaResult result = yerba_document_insert_object(document, StringValueCStr(path), StringValueCStr(json_string), before, after, at);
  check_result(result);
  return self;
}

/* document.insert_objects(path, array) */
static VALUE document_insert_objects(VALUE self, VALUE path, VALUE array) {
  struct Document *document = get_document(self);
  VALUE json_string = rb_funcall(rb_path2class("JSON"), rb_intern("generate"), 1, array);

  YerbaResult result = yerba_document_insert_objects(document, StringValueCStr(path), StringValueCStr(json_string));
  check_result(result);

  return self;
}

/* document.delete(path, condition: nil) */
static VALUE document_delete(int argc, VALUE *argv, VALUE self) {
  VALUE path, opts;
  rb_scan_args(argc, argv, "1:", &path, &opts);

  struct Document *document = get_document(self);
  if (!should_proceed(document, opts)) return self;

  YerbaResult result = yerba_document_delete(document, StringValueCStr(path));

  check_result(result);

  return self;
}

/* document.remove(path, value) */
static VALUE document_remove(VALUE self, VALUE path, VALUE value) {
  struct Document *document = get_document(self);
  YerbaResult result = yerba_document_remove(document, StringValueCStr(path), StringValueCStr(value));

  check_result(result);

  return self;
}

/* document.remove_at(path, index) */
static VALUE document_remove_at(VALUE self, VALUE path, VALUE index) {
  struct Document *document = get_document(self);
  YerbaResult result = yerba_document_remove_at(document, StringValueCStr(path), NUM2SIZET(index));

  check_result(result);

  return self;
}

/* document.rename(source, destination) */
static VALUE document_rename(VALUE self, VALUE source, VALUE destination) {
  struct Document *document = get_document(self);
  YerbaResult result = yerba_document_rename(document, StringValueCStr(source), StringValueCStr(destination));

  check_result(result);

  return self;
}

/* document.sort(path = "", by: nil, order: nil, case_sensitive: false) */
static VALUE document_sort(int argc, VALUE *argv, VALUE self) {
  VALUE path, opts;
  rb_scan_args(argc, argv, "01:", &path, &opts);

  if (NIL_P(path)) path = rb_str_new_cstr("");

  const char *by = NULL;
  bool case_sensitive = false;
  VALUE v_order = Qnil;

  if (!NIL_P(opts)) {
    VALUE v_by = rb_hash_aref(opts, ID2SYM(rb_intern("by")));
    v_order = rb_hash_aref(opts, ID2SYM(rb_intern("order")));
    VALUE v_case_sensitive = rb_hash_aref(opts, ID2SYM(rb_intern("case_sensitive")));

    if (SYMBOL_P(v_by)) {
      VALUE by_string = rb_sym2str(v_by);
      by = StringValueCStr(by_string);
    } else if (!NIL_P(v_by)) {
      by = StringValueCStr(v_by);
    }

    if (RTEST(v_case_sensitive)) {
      case_sensitive = true;
    }
  }

  struct Document *document = get_document(self);
  const char *path_string = StringValueCStr(path);

  if (RB_TYPE_P(v_order, T_ARRAY)) {
    VALUE order_csv = rb_ary_join(v_order, rb_str_new_cstr(","));
    const char *order_string = StringValueCStr(order_csv);
    const char *reorder_path = StringValueCStr(path);
    const char *reorder_by;

    if (by) {
      VALUE reorder_by_value = rb_hash_aref(opts, ID2SYM(rb_intern("by")));

      if (SYMBOL_P(reorder_by_value)) {
        reorder_by_value = rb_sym2str(reorder_by_value);
      }

      reorder_by = StringValueCStr(reorder_by_value);
    } else {
      reorder_by = ".";
    }

    YerbaResult result = yerba_document_reorder(document, reorder_path, reorder_by, order_string);
    check_result(result);

    return self;
  }

  const char *order = NULL;

  if (SYMBOL_P(v_order)) {
    VALUE order_string = rb_sym2str(v_order);
    order = StringValueCStr(order_string);
  } else if (!NIL_P(v_order)) {
    order = StringValueCStr(v_order);
  }

  VALUE by_with_order = Qnil;

  if (order && strcmp(order, "desc") == 0) {
    if (by) {
      by_with_order = rb_sprintf("%s:desc", by);
    } else {
      by_with_order = rb_str_new_cstr(":desc");
    }

    by = StringValueCStr(by_with_order);
  }

  YerbaResult result = yerba_document_sort(document, path_string, by, case_sensitive);

  check_result(result);

  return self;
}

/* document.sort_keys(path, order) */
static VALUE document_sort_keys(VALUE self, VALUE path, VALUE order) {
  struct Document *document = get_document(self);

  VALUE order_string;
  if (RB_TYPE_P(order, T_ARRAY)) {
    order_string = rb_ary_join(order, rb_str_new_cstr(","));
  } else {
    order_string = order;
  }

  YerbaResult result = yerba_document_sort_keys(document, StringValueCStr(path), StringValueCStr(order_string));

  check_result(result);

  return self;
}

/* document.quote_style(path: nil, key_style: nil, value_style: nil) */
static VALUE document_quote_style(int argc, VALUE *argv, VALUE self) {
  VALUE opts;
  rb_scan_args(argc, argv, ":", &opts);

  const char *path = NULL;
  const char *key_style = NULL;
  const char *value_style = NULL;

  if (!NIL_P(opts)) {
    VALUE v_path = rb_hash_aref(opts, ID2SYM(rb_intern("path")));
    VALUE v_key_style = rb_hash_aref(opts, ID2SYM(rb_intern("key_style")));
    VALUE v_value_style = rb_hash_aref(opts, ID2SYM(rb_intern("value_style")));

    if (!NIL_P(v_path)) path = StringValueCStr(v_path);
    if (!NIL_P(v_key_style)) key_style = StringValueCStr(v_key_style);
    if (!NIL_P(v_value_style)) value_style = StringValueCStr(v_value_style);
  }

  struct Document *document = get_document(self);
  YerbaResult result = yerba_document_quote_style(document, path, key_style, value_style);

  check_result(result);

  return self;
}

/* document.blank_lines(path, count) */
static VALUE document_blank_lines(VALUE self, VALUE path, VALUE count) {
  struct Document *document = get_document(self);
  YerbaResult result = yerba_document_blank_lines(document, StringValueCStr(path), NUM2SIZET(count));

  check_result(result);

  return self;
}

/* document.apply_yerbafile(yerbafile_path = nil) */
static VALUE document_apply_yerbafile(int argc, VALUE *argv, VALUE self) {
  VALUE yerbafile_path;
  rb_scan_args(argc, argv, "01", &yerbafile_path);

  struct Document *document = get_document(self);
  VALUE file_path = rb_iv_get(self, "@path");

  const char *file_path_str = NIL_P(file_path) ? "" : StringValueCStr(file_path);
  const char *yerbafile_path_str = NIL_P(yerbafile_path) ? NULL : StringValueCStr(yerbafile_path);

  YerbaResult result = yerba_document_apply_yerbafile(document, file_path_str, yerbafile_path_str);
  check_result(result);

  return self;
}

/* document.validate_schema(schema_json, selector = nil) */
static VALUE document_validate_schema(int argc, VALUE *argv, VALUE self) {
  VALUE schema_json, selector;
  rb_scan_args(argc, argv, "11", &schema_json, &selector);

  struct Document *document = get_document(self);
  const char *selector_str = NIL_P(selector) ? NULL : StringValueCStr(selector);

  char *result = yerba_document_validate_schema(document, StringValueCStr(schema_json), selector_str);

  if (!result) return rb_ary_new();

  VALUE json_string = make_utf8_string(result);
  yerba_string_free(result);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
}

/* document.to_s */
static VALUE document_to_s(VALUE self) {
  struct Document *document = get_document(self);
  char *content = yerba_document_to_string(document);
  VALUE string = make_utf8_string(content);

  yerba_string_free(content);

  return string;
}

/* document.save! */
static VALUE document_save(VALUE self) {
  VALUE path = rb_iv_get(self, "@path");

  if (NIL_P(path)) {
    rb_raise(rb_eError, "Cannot save: document has no file path");
  }

  VALUE content = document_to_s(self);

  rb_funcall(rb_cFile, rb_intern("write"), 2, path, content);

  return self;
}

/* document.changed? */
static VALUE document_changed_p(VALUE self) {
  VALUE path = rb_iv_get(self, "@path");
  if (NIL_P(path)) return Qtrue;

  VALUE current = document_to_s(self);
  VALUE original = rb_funcall(rb_cFile, rb_intern("read"), 1, path);

  return rb_str_equal(current, original) ? Qfalse : Qtrue;
}

/* document.location(selector = nil) → Yerba::Location */
static VALUE document_location(int argc, VALUE *argv, VALUE self) {
  VALUE path;
  rb_scan_args(argc, argv, "01", &path);

  struct Document *document = get_document(self);
  const char *selector = NIL_P(path) ? "" : StringValueCStr(path);

  YerbaGetResult result = yerba_document_get(document, selector);

  if (result.error) {
    yerba_get_result_free(result);
    return Qnil;
  }

  if (result.location.start_line == 0 && result.location.end_line == 0 && strlen(selector) > 0) {
    yerba_get_result_free(result);
    return Qnil;
  }

  VALUE location = location_to_ruby(result.location);
  yerba_get_result_free(result);

  return location;
}

/* document.path */
static VALUE document_path(VALUE self) {
  return rb_iv_get(self, "@path");
}

static VALUE located_list_to_ruby(YerbaTypedList result, VALUE document) {
  if (!result.json) return rb_ary_new();

  VALUE json_string = make_utf8_string(result.json);
  yerba_string_free(result.json);

  VALUE items = rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
  long length = RARRAY_LEN(items);
  VALUE array = rb_ary_new_capa(length);


  for (long i = 0; i < length; i++) {
    VALUE item = rb_ary_entry(items, i);
    VALUE node_type = rb_hash_aref(item, rb_str_new_cstr("node_type"));
    VALUE file_path = rb_hash_aref(item, rb_str_new_cstr("file_path"));
    VALUE selector = rb_hash_aref(item, rb_str_new_cstr("selector"));
    const char *type_str = NIL_P(node_type) ? "" : StringValueCStr(node_type);

    if (NIL_P(selector)) continue;

    VALUE line = rb_hash_aref(item, rb_str_new_cstr("line"));
    VALUE kwargs = rb_hash_new();
    VALUE location = rb_hash_aref(item, rb_str_new_cstr("location"));

    rb_hash_aset(kwargs, ID2SYM(rb_intern("selector")), selector);
    if (!NIL_P(file_path)) rb_hash_aset(kwargs, ID2SYM(rb_intern("file_path")), file_path);
    if (!NIL_P(line)) rb_hash_aset(kwargs, ID2SYM(rb_intern("line")), line);
    if (!NIL_P(location)) rb_hash_aset(kwargs, ID2SYM(rb_intern("location")), location);
    if (!NIL_P(document)) rb_hash_aset(kwargs, ID2SYM(rb_intern("document")), document);

    VALUE key_name = rb_hash_aref(item, rb_str_new_cstr("key_name"));

    if (!NIL_P(key_name)) {
      VALUE key_location = rb_hash_aref(item, rb_str_new_cstr("key_location"));
      VALUE key_kwargs = rb_hash_new();

      rb_hash_aset(key_kwargs, ID2SYM(rb_intern("selector")), Qnil);
      rb_hash_aset(key_kwargs, ID2SYM(rb_intern("value")), key_name);
      if (!NIL_P(key_location)) rb_hash_aset(key_kwargs, ID2SYM(rb_intern("location")), key_location);

      VALUE key_args[1] = { key_kwargs };
      VALUE key = rb_funcallv_kw(rb_path2class("Yerba::Scalar"), rb_intern("from"), 1, key_args, RB_PASS_KEYWORDS);

      rb_hash_aset(kwargs, ID2SYM(rb_intern("key")), key);
    }

    if (strcmp(type_str, "scalar") == 0) {
      VALUE text = rb_hash_aref(item, rb_str_new_cstr("text"));
      if (NIL_P(text)) continue;

      int type_value = NUM2INT(rb_hash_aref(item, rb_str_new_cstr("type")));
      YerbaTypedValue typed_value;
      typed_value.text = (char *)StringValueCStr(text);
      typed_value.value_type = (YerbaValueType)type_value;

      rb_hash_aset(kwargs, ID2SYM(rb_intern("value")), typed_value_to_ruby(typed_value));
      VALUE scalar_args[1] = { kwargs };
      rb_ary_push(array, rb_funcallv_kw(rb_path2class("Yerba::Scalar"), rb_intern("from"), 1, scalar_args, RB_PASS_KEYWORDS));
    } else {
      const char *klass_name = strcmp(type_str, "sequence") == 0 ? "Yerba::Sequence" : "Yerba::Map";
      VALUE node_args[1] = { kwargs };
      rb_ary_push(array, rb_funcallv_kw(rb_path2class(klass_name), rb_intern("from"), 1, node_args, RB_PASS_KEYWORDS));
    }
  }

  return array;
}

/* Collection.get(glob, selector) → [Yerba::Scalar|Map|Sequence, ...] */
static VALUE collection_s_get(VALUE self, VALUE pattern, VALUE path) {
  (void) self;

  return located_list_to_ruby(yerba_glob_get(StringValueCStr(pattern), StringValueCStr(path)), Qnil);
}

/* document.get_all(selector) → [Yerba::Scalar|Map|Sequence, ...] */
static VALUE document_get_all(VALUE self, VALUE path) {
  struct Document *document = get_document(self);

  return located_list_to_ruby(yerba_document_get_all(document, StringValueCStr(path)), self);
}

/* Collection.find(glob, path, condition: nil, select: nil) */
static VALUE collection_s_find(int argc, VALUE *argv, VALUE self) {
  (void)self;
  VALUE pattern, path, opts;
  rb_scan_args(argc, argv, "2:", &pattern, &path, &opts);

  const char *condition = NULL;
  const char *select = NULL;

  if (!NIL_P(opts)) {
    VALUE v_condition = rb_hash_aref(opts, ID2SYM(rb_intern("condition")));
    VALUE v_select = rb_hash_aref(opts, ID2SYM(rb_intern("select")));

    if (!NIL_P(v_condition)) condition = StringValueCStr(v_condition);
    if (!NIL_P(v_select)) select = StringValueCStr(v_select);
  }

  check_condition(condition, true);

  YerbaTypedList result = yerba_glob_find(StringValueCStr(pattern), StringValueCStr(path), condition, select);

  if (!result.json) return rb_ary_new();

  VALUE json_string = make_utf8_string(result.json);
  yerba_string_free(result.json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
}

/* Yerbafile.locate(directory = nil) → path string or nil */
static VALUE yerbafile_s_locate(int argc, VALUE *argv, VALUE klass) {
  VALUE directory;
  rb_scan_args(argc, argv, "01", &directory);

  const char *dir = NIL_P(directory) ? NULL : StringValueCStr(directory);
  char *result = yerba_yerbafile_find(dir);

  if (!result) return Qnil;

  VALUE path = make_utf8_string(result);
  yerba_string_free(result);

  return path;
}

void Init_yerba(void) {
  rb_require("json");

  rb_mYerba = rb_define_module("Yerba");
  rb_eError = rb_define_class_under(rb_mYerba, "Error", rb_eStandardError);
  rb_ePathNotFoundError = rb_define_class_under(rb_mYerba, "PathNotFoundError", rb_eError);
  rb_eParseError = rb_define_class_under(rb_mYerba, "ParseError", rb_eError);
  rb_ePathValidationError = rb_define_class_under(rb_mYerba, "PathValidationError", rb_eError);

  VALUE rb_cCollection = rb_define_class_under(rb_mYerba, "Collection", rb_cObject);
  rb_define_singleton_method(rb_cCollection, "get", collection_s_get, 2);
  rb_define_singleton_method(rb_cCollection, "find", collection_s_find, -1);

  rb_cDocument = rb_define_class_under(rb_mYerba, "Document", rb_cObject);

  rb_define_alloc_func(rb_cDocument, document_alloc);
  rb_define_method(rb_cDocument, "initialize", document_initialize, -1);
  rb_define_method(rb_cDocument, "replace_content!", document_replace_content, 1);
  rb_define_singleton_method(rb_cDocument, "parse", document_s_parse, 1);
  rb_define_method(rb_cDocument, "[]", document_bracket, 1);
  rb_define_method(rb_cDocument, "node_at", document_bracket, 1);
  rb_define_method(rb_cDocument, "value_at", document_get_value, 1);
  rb_define_method(rb_cDocument, "get_all", document_get_all, 1);
  rb_define_method(rb_cDocument, "location", document_location, -1);
  rb_define_method(rb_cDocument, "locations", document_locations, 1);
  rb_define_method(rb_cDocument, "selectors", document_selectors, 0);
  rb_define_method(rb_cDocument, "keys_at", document_keys_at, 1);
  rb_define_method(rb_cDocument, "resolve_selectors", document_resolve_selectors, 1);
  rb_define_method(rb_cDocument, "get_quote_style", document_get_quote_style, 1);
  rb_define_method(rb_cDocument, "set_quote_style", document_set_quote_style, 2);
  rb_define_method(rb_cDocument, "get_collection_style", document_get_collection_style, 1);
  rb_define_method(rb_cDocument, "set_collection_style", document_set_collection_style, 2);
  rb_define_method(rb_cDocument, "get_sequence_indent", document_get_sequence_indent, 1);
  rb_define_method(rb_cDocument, "set_sequence_indent", document_set_sequence_indent, 2);
  rb_define_method(rb_cDocument, "exists?", document_exists_p, 1);
  rb_define_method(rb_cDocument, "valid_selector?", document_valid_selector_p, 1);
  rb_define_method(rb_cDocument, "condition?", document_condition_p, -1);
  rb_define_method(rb_cDocument, "find", document_find, -1);
  rb_define_method(rb_cDocument, "set", document_set, -1);
  rb_define_method(rb_cDocument, "insert", document_insert, -1);
  rb_define_method(rb_cDocument, "insert_object", document_insert_object, -1);
  rb_define_method(rb_cDocument, "insert_objects", document_insert_objects, 2);
  rb_define_method(rb_cDocument, "delete", document_delete, -1);
  rb_define_method(rb_cDocument, "remove", document_remove, 2);
  rb_define_method(rb_cDocument, "remove_at", document_remove_at, 2);
  rb_define_method(rb_cDocument, "rename", document_rename, 2);
  rb_define_method(rb_cDocument, "sort", document_sort, -1);
  rb_define_method(rb_cDocument, "sort_keys", document_sort_keys, 2);
  rb_define_method(rb_cDocument, "quote_style", document_quote_style, -1);
  rb_define_method(rb_cDocument, "blank_lines", document_blank_lines, 2);
  rb_define_method(rb_cDocument, "apply_yerbafile", document_apply_yerbafile, -1);
  rb_define_method(rb_cDocument, "validate_schema", document_validate_schema, -1);
  rb_define_method(rb_cDocument, "source", document_source, 1);
  rb_define_method(rb_cDocument, "to_s", document_to_s, 0);
  rb_define_method(rb_cDocument, "write!", document_save, 0);
  rb_define_method(rb_cDocument, "changed?", document_changed_p, 0);
  rb_define_method(rb_cDocument, "path", document_path, 0);

  VALUE rb_cYerbafile = rb_define_class_under(rb_mYerba, "Yerbafile", rb_cObject);
  rb_define_singleton_method(rb_cYerbafile, "locate", yerbafile_s_locate, -1);
}
