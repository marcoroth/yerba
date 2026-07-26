# frozen_string_literal: true

require "test_helper"

class DocumentConditionValidationTest < Minitest::Spec
  ITEMS = <<~YAML
    items:
      - name: "a"
        kind: "keynote"
      - name: "b"
        kind: "talk"
  YAML

  def document
    @document ||= Yerba::Document.parse(ITEMS)
  end

  test "find accepts a well-formed condition" do
    assert_equal [{ "name" => "a", "kind" => "keynote" }], document.find("items[]", condition: '.kind == "keynote"')
  end

  test "find still returns nothing for a condition that matches nothing" do
    assert_empty document.find("items[]", condition: '.kind == "workshop"')
  end

  test "find raises on a condition using a single equals" do
    error = assert_raises(Yerba::Error) { document.find("items[]", condition: '.kind = "keynote"') }

    assert_includes error.message, "invalid condition"
    assert_includes error.message, "=="
  end

  test "find raises on a condition missing the leading dot" do
    error = assert_raises(Yerba::Error) { document.find("items[]", condition: 'kind == "keynote"') }

    assert_includes error.message, "must be relative"
    assert_includes error.message, '".kind"'
  end

  test "find raises on a condition with no operator" do
    assert_raises(Yerba::Error) { document.find("items[]", condition: "garbage!!") }
  end

  test "find raises on an empty condition" do
    assert_raises(Yerba::Error) { document.find("items[]", condition: "") }
  end

  test "find raises on a condition with no selector" do
    assert_raises(Yerba::Error) { document.find("items[]", condition: ". == keynote") }
  end

  test "condition? accepts a relative condition" do
    assert Yerba::Document.parse("host: localhost\n").condition?(".host == localhost")
  end

  test "condition? accepts an absolute condition when there is no parent" do
    assert Yerba::Document.parse("host: localhost\n").condition?("host == localhost")
  end

  test "condition? raises on a malformed condition" do
    assert_raises(Yerba::Error) { Yerba::Document.parse("host: localhost\n").condition?("host = localhost") }
  end

  test "set applies a well-formed condition" do
    target = Yerba::Document.parse("port: 5432\nhost: old\n")
    target.set("host", "new", condition: ".port == 5432")

    assert_equal "new", target.value_at("host")
  end

  test "set leaves the document alone when a well-formed condition does not match" do
    target = Yerba::Document.parse("port: 1234\nhost: old\n")
    target.set("host", "new", condition: ".port == 5432")

    assert_equal "old", target.value_at("host")
  end

  test "set raises rather than silently skipping on a malformed condition" do
    target = Yerba::Document.parse("port: 5432\nhost: old\n")

    assert_raises(Yerba::Error) { target.set("host", "new", condition: ".port = 5432") }
    assert_equal "old", target.value_at("host")
  end

  test "delete raises rather than silently skipping on a malformed condition" do
    target = Yerba::Document.parse("port: 5432\nhost: old\n")

    assert_raises(Yerba::Error) { target.delete("host", condition: ".port = 5432") }
    assert_equal "old", target.value_at("host")
  end
end
