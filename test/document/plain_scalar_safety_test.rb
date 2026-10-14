# frozen_string_literal: true

require "test_helper"
require "psych"

class PlainScalarSafetyTest < Minitest::Spec
  HAZARDOUS = [
    "One Record: Concurrency",
    "trailing colon:",
    "#hashstart",
    "has # hash",
    "  padded  ",
    "*alias",
    "&anchor",
    "a, b",
    'say "hi": now',
    "back\\slash: x"
  ].freeze

  test "set over a plain scalar quotes a value that cannot be written plain" do
    document = Yerba::Document.parse(<<~YAML)
      raw_title: plainvalue
    YAML

    document.set("raw_title", "One Record: Concurrency")

    assert_includes document.to_s, 'raw_title: "One Record: Concurrency"'
  end

  test "set over a plain scalar leaves a safe value plain" do
    document = Yerba::Document.parse(<<~YAML)
      raw_title: plainvalue
    YAML

    document.set("raw_title", "Why Git Still Matters")

    assert_includes document.to_s, "raw_title: Why Git Still Matters"
    refute_includes document.to_s, '"Why Git Still Matters"'
  end

  test "set still preserves an existing quote style" do
    document = Yerba::Document.parse(<<~YAML)
      raw_title: "quoted"
    YAML

    document.set("raw_title", "One Record: Concurrency")

    assert_includes document.to_s, 'raw_title: "One Record: Concurrency"'
  end

  test "insert quotes a map value that cannot be written plain" do
    document = Yerba::Document.parse(<<~YAML)
      talk:
        title: Two Requests
    YAML

    document.insert("talk.raw_title", "One Record: Concurrency")

    assert_includes document.to_s, 'raw_title: "One Record: Concurrency"'
  end

  test "set round-trips hazardous strings through Psych" do
    HAZARDOUS.each do |value|
      document = Yerba::Document.parse(<<~YAML)
        raw_title: plainvalue
      YAML

      document.set("raw_title", value)
      output = document.to_s

      loaded = begin
        Psych.load(output)
      rescue Psych::SyntaxError => e
        flunk "Psych could not parse output for #{value.inspect}: #{e.message}\n#{output}"
      end

      assert_equal value, loaded["raw_title"], "value #{value.inspect} did not round-trip; document was:\n#{output}"
      assert_equal value, document.value_at("raw_title"), "yerba read back #{value.inspect} differently"
    end
  end

  test "insert round-trips hazardous strings through Psych" do
    HAZARDOUS.each do |value|
      document = Yerba::Document.parse(<<~YAML)
        talk:
          title: Two Requests
      YAML

      document.insert("talk.raw_title", value)
      output = document.to_s

      loaded = begin
        Psych.load(output)
      rescue Psych::SyntaxError => e
        flunk "Psych could not parse output for #{value.inspect}: #{e.message}\n#{output}"
      end

      assert_equal value, loaded.dig("talk", "raw_title"), "value #{value.inspect} did not round-trip; document was:\n#{output}"
    end
  end

  test "set preserves a string that looks like another type" do
    document = Yerba::Document.parse(<<~YAML)
      flag: placeholder
    YAML

    document.set("flag", "true")

    assert_includes document.to_s, 'flag: "true"'
    assert_equal "true", Psych.load(document.to_s)["flag"]
  end

  test "set writes a typed value as a YAML literal" do
    document = Yerba::Document.parse(<<~YAML)
      flag: placeholder
      count: placeholder
      replica: placeholder
    YAML

    document.set("flag", true)
    document.set("count", 12_345)
    document.set("replica", nil)

    loaded = Psych.load(document.to_s)

    assert_equal true, loaded["flag"]
    assert_equal 12_345, loaded["count"]
    assert_nil loaded["replica"]
  end

  test "set preserves a string regardless of what the field held before" do
    document = Yerba::Document.parse(<<~YAML)
      flag: false
      count: 5
    YAML

    document.set("flag", "true")
    document.set("count", "10")

    loaded = Psych.load(document.to_s)

    assert_equal "true", loaded["flag"]
    assert_equal "10", loaded["count"]
  end

  test "insert does preserve a string that looks like another type" do
    document = Yerba::Document.parse(<<~YAML)
      talk:
        title: Two Requests
    YAML

    document.insert("talk.flag", "true")

    assert_includes document.to_s, 'flag: "true"'
    assert_equal "true", Psych.load(document.to_s).dig("talk", "flag")
  end

  test "a leading hyphen is not treated as a sequence indicator" do
    document = Yerba::Document.parse(<<~YAML)
      video_id: placeholder
    YAML

    document.set("video_id", "-dZZJ6pex-g")

    assert_includes document.to_s, "video_id: -dZZJ6pex-g"
    assert_equal "-dZZJ6pex-g", Psych.load(document.to_s)["video_id"]
  end

  test "set over a scalar quotes a dash-prefixed fragment instead of emitting invalid YAML" do
    document = Yerba::Document.parse(<<~YAML)
      tags: old
    YAML

    document.set("tags", "- new")

    output = document.to_s

    assert_includes output, 'tags: "- new"'

    loaded = begin
      Psych.load(output)
    rescue Psych::SyntaxError => e
      flunk "Psych could not parse output: #{e.message}\n#{output}"
    end

    assert_equal "- new", loaded["tags"]
  end

  PSYCH_NON_STRINGS = [
    "12:30:00", "-1:30:15.5", "0755", "0b1010", "1_000", "1,000", "1_000.5", "2025-01-01",
    "2025-01-01T10:00:00Z", ":symbol", "tRuE", "YeS", "nUlL"
  ].freeze

  test "strings Psych would load as another type are quoted by every writer" do
    PSYCH_NON_STRINGS.each do |string|
      document = Yerba::Document.parse(<<~YAML)
        value: x
        list:
          - a
      YAML

      document.set("value", string)
      document.insert("inserted", string)
      document.root["block"] = [string]
      document.root.set("flow", [string], style: :flow)
      items = [string]
      document["list"].concat(items)

      loaded = Psych.unsafe_load(document.to_s)

      assert_equal string, loaded["value"], string
      assert_equal string, loaded["inserted"], string
      assert_equal [string], loaded["block"], string
      assert_equal [string], loaded["flow"], string
      assert_equal string, loaded["list"].last, string
      assert_equal({ "value" => string }, Psych.unsafe_load(Yerba::Document.from({ "value" => string }).to_s), string)
    end
  end

  test "flow collections quote strings that would change type or structure" do
    document = Yerba::Document.parse("tags: x\n")

    document.root.set("tags", ["yes", "x, y", "a: b", "plain"], style: :flow)

    assert_equal %(tags: ["yes", "x, y", "a: b", plain]\n), document.to_s
  end
end
