# frozen_string_literal: true

require "test_helper"
require "psych"

module Document
  class PsychTypesTest < Minitest::Spec
    CHANGES_TYPE = [
      "0755", "-0755", "00", "0_7", "0b1010", "-0b101", "0x1F", "+0x1F", "1_000", "1,000", "-1_000",
      "12:30:00", "1:30", "-1:30", "1_0:30", "1:30.5", "-1:30.5", "1:30.", "190:20:30.15",
      "1.5", "1_000.5", "1,000.5", ".5", "-.5", "1.", "1.e+5", "1.5E-3", "9223372036854775808",
      ".inf", ".Inf", ".iNf", "+.inf", "-.inf", ".nan", ".NaN", ".nAn",
      "yes", "yEs", "tRuE", "fAlSe", "on", "oN", "OFF", "null", "nUlL", "~"
    ].freeze

    STAYS_STRING = [
      "08", "09", "0o17", "0O17", "0X1F", "0B101", "0x", "1__0", "1_000_", "1,,0", "1:60", "12:3:45:00",
      "1.0e5", "1e5", "1E+5", ".", "-.", "1..5", "1.2.3", "v1.0", "0.1_5", "inf", "nan", "NaN", "Infinity",
      "y", "n", "yes!", "Yesss", "1a", "a1"
    ].freeze

    def assert_same_value(expected, actual, message)
      if expected.nil?
        assert_nil actual, message
      elsif expected.is_a?(Float) && expected.nan?
        assert actual.is_a?(Float) && actual.nan?, message
      else
        assert_equal expected, actual, message
        assert_equal expected.class, actual.class, message
      end
    end

    def assert_reads_like_psych(scalar)
      yaml = "value: #{scalar}\n"
      expected = Psych.unsafe_load(yaml)["value"]
      document = Yerba::Document.parse(yaml)

      assert_same_value expected, document.value_at("value"), "value_at(#{scalar.inspect})"
      assert_same_value expected, document.values["value"], "values for #{scalar.inspect}"
      assert_same_value expected, document["value"].value, "node value for #{scalar.inspect}"
    end

    test "plain scalars that Psych reads as numbers, booleans or nil read the same" do
      CHANGES_TYPE.each { |scalar| assert_reads_like_psych(scalar) }
    end

    test "plain scalars that Psych reads as strings stay strings" do
      STAYS_STRING.each do |scalar|
        assert_reads_like_psych(scalar)
        assert_equal scalar, Yerba::Document.parse("value: #{scalar}\n").value_at("value")
      end
    end

    test "issue examples read the way Psych reads them" do
      assert_equal 493, Yerba.parse("mode: 0755\n").value_at("mode")
      assert_equal 45_000, Yerba.parse("time: 12:30:00\n").value_at("time")
    end

    test "dates, times and symbols read as strings" do
      document = Yerba::Document.parse(<<~YAML)
        date: 2025-01-01
        time: 2025-01-01T10:00:00Z
        symbol: :foo
      YAML

      assert_equal "2025-01-01", document.value_at("date")
      assert_equal "2025-01-01T10:00:00Z", document.value_at("time")
      assert_equal ":foo", document.value_at("symbol")
    end

    test "quoted scalars stay strings" do
      document = Yerba::Document.parse(<<~YAML)
        mode: "0755"
        time: '12:30:00'
      YAML

      assert_equal({ "mode" => "0755", "time" => "12:30:00" }, document.values)
    end

    test "node records, get_all and to_h agree on the converted values" do
      document = Yerba::Document.parse(<<~YAML)
        - 0755
        - 1:30.5
        - .Inf
        - tRuE
        - 08
      YAML

      expected = [493, 5430.0, Float::INFINITY, true, "08"]

      assert_equal expected, document.to_a
      assert_equal expected, document.root.map(&:value)
      assert_equal expected, document.get_all("[]").map(&:value)
    end

    test "setting a string that reads as a number quotes it" do
      document = Yerba::Document.parse("mode: plain\n")

      document.set("mode", "0755")

      assert_equal "mode: \"0755\"\n", document.to_s
      assert_equal "0755", document.value_at("mode")
      assert_equal "0755", Psych.unsafe_load(document.to_s)["mode"]
    end
  end
end
