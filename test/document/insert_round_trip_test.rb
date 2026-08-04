# frozen_string_literal: true

require "test_helper"
require "yaml"

class DocumentInsertRoundTripTest < Minitest::Spec
  SHAPES = {
    "multi-line" => "line one\nline two",
    "already quoted" => '"already"',
    "sequence-like" => "- not a list",
    "flow-like" => "[a, b]",
    "map-like" => "key: value",
    "tab" => "a\tb",
    "number-like" => "12345",
  }.freeze

  SHAPES.each do |name, text|
    test "Document#insert keeps a #{name} string intact" do
      document = Yerba.parse("---\nexisting: x\n")
      document.insert("desc", text)

      assert_equal text, YAML.safe_load(document.to_s)["desc"]
    end

    test "Map#[]= keeps a #{name} string intact" do
      document = Yerba.parse("---\nexisting: x\n")
      document["desc"] = text

      assert_equal text, YAML.safe_load(document.to_s)["desc"]
    end

    test "Map#insert keeps a #{name} string intact" do
      document = Yerba.parse("---\nexisting: x\n")
      document.root.insert("desc", text)

      assert_equal text, YAML.safe_load(document.to_s)["desc"]
    end

    test "Sequence#<< keeps a #{name} string intact" do
      document = Yerba.parse("---\nitems:\n  - first\n")
      document["items"] << text

      assert_equal ["first", text], YAML.safe_load(document.to_s)["items"]
    end
  end

  test "a value is quoted once, not once per layer" do
    document = Yerba.parse("---\nexisting: x\n")
    document["desc"] = '"already"'

    refute_includes document.to_s, "\\\\"
    assert_equal '"already"', YAML.safe_load(document.to_s)["desc"]
  end

  test "a collection still goes in as a collection, not as quoted text" do
    document = Yerba.parse("---\nexisting: x\n")
    document["tags"] = ["ruby", "rails"]

    assert_equal ["ruby", "rails"], YAML.safe_load(document.to_s)["tags"]
  end

  test "an explicitly plain value is inserted as YAML text" do
    document = Yerba.parse("---\nexisting: x\n")
    document.insert("tags", "- ruby\n- rails", plain: true)

    assert_equal ["ruby", "rails"], YAML.safe_load(document.to_s)["tags"]
  end

  test "a new sequence entry matches the quote style of its siblings" do
    document = Yerba.parse(%(---\nitems:\n  - "first"\n))
    document["items"] << "second"

    assert_includes document.to_s, %("second")
  end

  test "a string containing a NUL reaches the escaper" do
    document = Yerba.parse("---\nexisting: x\n")
    document["desc"] = "a\0b"

    assert_equal "a\0b", YAML.safe_load(document.to_s)["desc"]
  end
end
