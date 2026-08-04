# frozen_string_literal: true

require "test_helper"
require "yaml"

class DocumentBuildScalarTest < Minitest::Spec
  MULTILINE = "line one\nline two\n\nline four"

  test "a multi-line string assigned to a root key round-trips" do
    document = Yerba::Document.new
    document.root = {}
    document["desc"] = MULTILINE

    assert_equal MULTILINE, YAML.safe_load(document.to_s)["desc"]
  end

  test "a multi-line string appended inside a sequence round-trips" do
    document = Yerba::Document.new
    document.root = []
    document << { "id" => "x", "talks" => [{ "id" => "t", "desc" => MULTILINE }] }

    assert_equal MULTILINE, YAML.safe_load(document.to_s).dig(0, "talks", 0, "desc")
  end

  test "a multi-line string round-trips through Document.from" do
    document = Yerba::Document.from([{ "id" => "a", "desc" => MULTILINE }])

    assert_equal MULTILINE, YAML.safe_load(document.to_s).dig(0, "desc")
  end

  test "a string keeps its line breaks rather than becoming a sequence" do
    document = Yerba::Document.new
    document.root = {}
    document["desc"] = "- a\n- b"

    assert_equal "- a\n- b", YAML.safe_load(document.to_s)["desc"]
  end

  test "tabs and control characters round-trip" do
    document = Yerba::Document.new
    document.root = {}
    document["desc"] = "a\tbc"

    assert_equal "a\tbc", YAML.safe_load(document.to_s)["desc"]
  end

  test "an empty sequence reads back as an empty sequence, not null" do
    document = Yerba::Document.new
    document.root = []
    document << { "id" => "t", "speakers" => [], "date" => "x" }

    assert_equal [], YAML.safe_load(document.to_s).dig(0, "speakers")
  end

  test "an empty mapping reads back as an empty mapping, not null" do
    document = Yerba::Document.new
    document.root = []
    document << { "id" => "t", "meta" => {}, "date" => "x" }

    assert_equal({}, YAML.safe_load(document.to_s).dig(0, "meta"))
  end

  test "a value that looks like a number stays a string" do
    document = Yerba::Document.new
    document.root = []
    document << { "id" => "t", "version" => "12345" }

    assert_equal "12345", YAML.safe_load(document.to_s).dig(0, "version")
  end

  test "an empty sequence under a key survives Document.from" do
    document = Yerba::Document.from({ "talks" => [], "id" => "x" })

    assert_equal({ "talks" => [], "id" => "x" }, YAML.safe_load(document.to_s))
  end

  test "an empty mapping under a key survives Document.from" do
    document = Yerba::Document.from({ "meta" => {}, "id" => "x" })

    assert_equal({ "meta" => {}, "id" => "x" }, YAML.safe_load(document.to_s))
  end

  test "an empty collection nested in a sequence survives Document.from" do
    document = Yerba::Document.from([{ "id" => "t", "speakers" => [] }])

    assert_equal([{ "id" => "t", "speakers" => [] }], YAML.safe_load(document.to_s))
  end
end
