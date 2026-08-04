# frozen_string_literal: true

require "test_helper"
require "yaml"

class DocumentBlockScalarIndentTest < Minitest::Spec
  MULTILINE = "line one\nline two"

  test "converting to literal works at the document root" do
    document = Yerba.parse(%(---\ndesc: "line one\\nline two"\n))
    document.quote_style(path: "desc", value_style: "literal")

    assert_equal MULTILINE, YAML.safe_load(document.to_s)["desc"]
  end

  test "converting to literal works inside a sequence item" do
    document = Yerba.parse(%(---\n- desc: "line one\\nline two"\n))
    document.quote_style(path: "[].desc", value_style: "literal")

    assert_equal MULTILINE, YAML.safe_load(document.to_s).dig(0, "desc")
  end

  test "converting to literal works inside a nested sequence item" do
    document = Yerba.parse(%(---\n- talks:\n    - desc: "line one\\nline two"\n))
    document.quote_style(path: "[].talks[].desc", value_style: "literal")

    assert_equal MULTILINE, YAML.safe_load(document.to_s).dig(0, "talks", 0, "desc")
  end

  test "escaped characters are decoded into the block body" do
    document = Yerba.parse(%(---\n- desc: "a\\nb \\U0001F600"\n))
    document.quote_style(path: "[].desc", value_style: "literal")

    assert_equal "a\nb 😀", YAML.safe_load(document.to_s).dig(0, "desc")
  end

  test "a block scalar converted back to double keeps its line breaks" do
    document = Yerba.parse(%(---\n- desc: |-\n    line one\n    line two\n))
    document.quote_style(path: "[].desc", value_style: "double")

    assert_equal MULTILINE, YAML.safe_load(document.to_s).dig(0, "desc")
  end
end
