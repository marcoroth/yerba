# frozen_string_literal: true

require "test_helper"

class DocumentKeysAtTest < Minitest::Spec
  test "keys_at returns the keys of the document root" do
    document = Yerba::Document.parse(<<~YAML)
      name: Conf
      year: 2024
      venue:
        city: Berlin
    YAML

    assert_equal ["name", "year", "venue"], document.keys_at("")
  end

  test "keys_at returns the keys of a nested map" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal ["host", "port"], document.keys_at("database")
  end

  test "keys_at does not descend into a nested sequence of maps" do
    document = Yerba::Document.parse(<<~YAML)
      id: "aloha"
      name: "Aloha"
      channels:
        - id: "UC123"
          handle: "@confreaks"
    YAML

    assert_equal ["id", "name", "channels"], document.keys_at("")
  end

  test "keys_at does not descend into a nested sequence of scalars" do
    document = Yerba::Document.parse(<<~YAML)
      name: Conf
      tags:
        - ruby
        - rails
    YAML

    assert_equal ["name", "tags"], document.keys_at("")
  end

  test "keys_at reports a key whose value is empty" do
    document = Yerba::Document.parse(<<~YAML)
      a: 1
      b:
      c: 3
    YAML

    assert_equal ["a", "b", "c"], document.keys_at("")
  end

  test "keys_at reads the keys of a flow map" do
    document = Yerba::Document.parse("venue: { city: Berlin, country: DE }\n")

    assert_equal ["city", "country"], document.keys_at("venue")
  end

  test "keys_at reads the keys of a map inside a sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - id: "a"
        title: "First"
      - id: "b"
    YAML

    assert_equal ["id", "title"], document.keys_at("[0]")
  end

  test "keys_at returns nothing for a sequence" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML

    assert_empty document.keys_at("tags")
  end

  test "keys_at returns nothing for a sequence document root" do
    document = Yerba::Document.parse(<<~YAML)
      - id: "a"
      - id: "b"
    YAML

    assert_empty document.keys_at("")
  end

  test "keys_at returns nothing for a scalar" do
    assert_empty Yerba::Document.parse("name: Conf\n").keys_at("name")
  end

  test "keys_at returns nothing for a selector that matches nothing" do
    assert_empty Yerba::Document.parse("name: Conf\n").keys_at("missing")
  end

  test "keys_at matches the keys of the parsed value" do
    document = Yerba::Document.parse(<<~YAML)
      name: Conf
      venue:
        city: Berlin
        country: DE
    YAML

    ["", "venue"].each do |selector|
      assert_equal document.value_at(selector).keys, document.keys_at(selector), selector
    end
  end
end
