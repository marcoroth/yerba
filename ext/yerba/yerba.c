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

static int should_proceed(struct Document *document, VALUE opts) {
  if (NIL_P(opts)) return 1;
  VALUE v_condition = rb_hash_aref(opts, ID2SYM(rb_intern("condition")));
  if (NIL_P(v_condition)) return 1;

  VALUE v_path = rb_hash_aref(opts, ID2SYM(rb_intern("condition_path")));
  const char *parent_path = NIL_P(v_path) ? "" : StringValueCStr(v_path);

  return yerba_document_evaluate_condition(document, parent_path, StringValueCStr(v_condition));
}

/* Document.new(path) */
static VALUE document_initialize(VALUE self, VALUE path) {
  const char *file_path = StringValueCStr(path);
  YerbaParseResult result = yerba_document_parse_file(file_path);

  if (!result.document) {
    VALUE message = make_utf8_string(result.error);
    yerba_string_free(result.error);

    rb_raise(rb_eParseError, "%s", StringValueCStr(message));
  }

  RTYPEDDATA_DATA(self) = result.document;
  rb_iv_set(self, "@path", path);

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

/* document.get(path) */
static VALUE document_get(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  YerbaGetResult result = yerba_document_get(document, StringValueCStr(path));

  if (result.error) {
    VALUE message = make_utf8_string(result.error);
    yerba_get_result_free(result);

    rb_raise(rb_ePathValidationError, "%s", StringValueCStr(message));
  }

  if (!result.is_list) {
    VALUE ruby_value = typed_value_to_ruby(result.single);
    yerba_get_result_free(result);

    return ruby_value;
  } else {
    VALUE json_string = make_utf8_string(result.list.json);
    yerba_get_result_free(result);

    VALUE items = rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
    long length = RARRAY_LEN(items);
    VALUE array = rb_ary_new_capa(length);

    for (long i = 0; i < length; i++) {
      VALUE item = rb_ary_entry(items, i);
      VALUE text = rb_hash_aref(item, rb_str_new_cstr("text"));
      int type_value = NUM2INT(rb_hash_aref(item, rb_str_new_cstr("type")));

      if (NIL_P(text)) {
        rb_ary_push(array, Qnil);

        continue;
      }

      YerbaTypedValue typed_value;
      typed_value.text = (char *)StringValueCStr(text);
      typed_value.value_type = (YerbaValueType)type_value;

      rb_ary_push(array, typed_value_to_ruby(typed_value));
    }

    return array;
  }
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

/* document[](path) → Yerba::Scalar, Yerba::Map, Yerba::Sequence, or nil */
static VALUE document_bracket(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  YerbaGetResult result = yerba_document_get(document, StringValueCStr(path));

  if (result.error) {
    VALUE message = make_utf8_string(result.error);
    yerba_get_result_free(result);

    rb_raise(rb_ePathValidationError, "%s", StringValueCStr(message));
  }

  VALUE instance;
  VALUE location = location_to_ruby(result.location);
  VALUE key = Qnil;

  if (result.key_name) {
    VALUE key_location = location_to_ruby(result.key_location);
    VALUE key_value = make_utf8_string(result.key_name);

    key = rb_funcall(rb_path2class("Yerba::Scalar"), rb_intern("new"), 4, Qnil, Qnil, key_value, key_location);
  }

  switch (result.node_type) {
    case NODE_TYPE_SCALAR: {
      VALUE klass = rb_path2class("Yerba::Scalar");
      VALUE value = typed_value_to_ruby(result.single);
      yerba_get_result_free(result);

      instance = rb_funcall(klass, rb_intern("new"), 5, self, path, value, location, key);

      return instance;
    }

    case NODE_TYPE_MAP: {
      yerba_get_result_free(result);
      VALUE klass = rb_path2class("Yerba::Map");

      instance = rb_funcall(klass, rb_intern("new"), 4, self, path, location, key);

      return instance;
    }

    case NODE_TYPE_SEQUENCE: {
      yerba_get_result_free(result);
      VALUE klass = rb_path2class("Yerba::Sequence");

      instance = rb_funcall(klass, rb_intern("new"), 4, self, path, location, key);

      return instance;
    }

    default:
      yerba_get_result_free(result);

      return Qnil;
  }
}

/* document.exists?(path) */
static VALUE document_exists_p(VALUE self, VALUE path) {
  struct Document *document = get_document(self);

  return yerba_document_exists(document, StringValueCStr(path)) ? Qtrue : Qfalse;
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

/* document.get_values(path) → Array of parsed Ruby objects */
static VALUE document_get_values(VALUE self, VALUE path) {
  struct Document *document = get_document(self);
  char *json = yerba_document_get_values(document, StringValueCStr(path));

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
  char *json = yerba_document_find(document, StringValueCStr(path), condition, select);

  if (!json) return rb_ary_new();

  VALUE json_string = make_utf8_string(json);
  yerba_string_free(json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
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

  const char *c_value;
  YerbaValueType value_type;
  char number_buffer[64];
  bool all = false;

  if (value == Qnil) {
    c_value = "null";
    value_type = YERBA_VALUE_TYPE_NULL;
  } else if (value == Qtrue) {
    c_value = "true";
    value_type = YERBA_VALUE_TYPE_BOOLEAN;
  } else if (value == Qfalse) {
    c_value = "false";
    value_type = YERBA_VALUE_TYPE_BOOLEAN;
  } else if (RB_INTEGER_TYPE_P(value)) {
    snprintf(number_buffer, sizeof(number_buffer), "%ld", NUM2LONG(value));
    c_value = number_buffer;
    value_type = YERBA_VALUE_TYPE_INTEGER;
  } else if (RB_FLOAT_TYPE_P(value)) {
    snprintf(number_buffer, sizeof(number_buffer), "%g", NUM2DBL(value));
    c_value = number_buffer;
    value_type = YERBA_VALUE_TYPE_FLOAT;
  } else {
    c_value = StringValueCStr(value);
    value_type = YERBA_VALUE_TYPE_STRING;
  }

  if (!NIL_P(opts)) {
    VALUE v_all = rb_hash_aref(opts, ID2SYM(rb_intern("all")));

    if (RTEST(v_all)) all = true;
  }

  YerbaResult result = yerba_document_set(document, StringValueCStr(path), c_value, value_type, all);
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

  struct Document *document = get_document(self);
  YerbaResult result = yerba_document_insert(document, StringValueCStr(path), StringValueCStr(value), before, after, at);
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

/* document.path */
static VALUE document_path(VALUE self) {
  return rb_iv_get(self, "@path");
}

/* Collection.get(glob, path) — get values across files */
static VALUE collection_s_get(VALUE self, VALUE pattern, VALUE path) {
  (void)self;
  YerbaTypedList result = yerba_glob_get(StringValueCStr(pattern), StringValueCStr(path));

  if (!result.json) return rb_ary_new();

  VALUE json_string = make_utf8_string(result.json);
  yerba_string_free(result.json);

  VALUE items = rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
  long length = RARRAY_LEN(items);
  VALUE array = rb_ary_new_capa(length);

  for (long i = 0; i < length; i++) {
    VALUE item = rb_ary_entry(items, i);
    VALUE text = rb_hash_aref(item, rb_str_new_cstr("text"));
    int type_value = NUM2INT(rb_hash_aref(item, rb_str_new_cstr("type")));

    if (NIL_P(text)) {
      rb_ary_push(array, Qnil);
      continue;
    }

    YerbaTypedValue typed_value;
    typed_value.text = (char *)StringValueCStr(text);
    typed_value.value_type = (YerbaValueType)type_value;
    rb_ary_push(array, typed_value_to_ruby(typed_value));
  }

  return array;
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

  YerbaTypedList result = yerba_glob_find(StringValueCStr(pattern), StringValueCStr(path), condition, select);

  if (!result.json) return rb_ary_new();

  VALUE json_string = make_utf8_string(result.json);
  yerba_string_free(result.json);

  return rb_funcall(rb_path2class("JSON"), rb_intern("parse"), 1, json_string);
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
  rb_define_method(rb_cDocument, "initialize", document_initialize, 1);
  rb_define_singleton_method(rb_cDocument, "parse", document_s_parse, 1);
  rb_define_method(rb_cDocument, "get", document_get, 1);
  rb_define_method(rb_cDocument, "[]", document_bracket, 1);
  rb_define_method(rb_cDocument, "get_value", document_get_value, 1);
  rb_define_method(rb_cDocument, "get_values", document_get_values, 1);
  rb_define_method(rb_cDocument, "get_quote_style", document_get_quote_style, 1);
  rb_define_method(rb_cDocument, "set_quote_style", document_set_quote_style, 2);
  rb_define_method(rb_cDocument, "exists?", document_exists_p, 1);
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
  rb_define_method(rb_cDocument, "to_s", document_to_s, 0);
  rb_define_method(rb_cDocument, "save!", document_save, 0);
  rb_define_method(rb_cDocument, "changed?", document_changed_p, 0);
  rb_define_method(rb_cDocument, "path", document_path, 0);
}
