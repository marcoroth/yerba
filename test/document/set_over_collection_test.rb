# frozen_string_literal: true

require "test_helper"

class DocumentSetOverCollectionTest < Minitest::Spec
  test "assigning a string over a map replaces the map" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: "Berlin"
        country: "DE"
      name: "x"
    YAML

    document.root["venue"] = "none"

    assert_equal "venue: none\nname: \"x\"\n", document.to_s
    assert_equal "none", document.value_at("venue")
  end

  test "assigning over a map does not rename its first key" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: "Berlin"
      name: "x"
    YAML

    document.root["venue"] = "none"

    refute_includes document.to_s, "none: "
    refute_includes document.keys_at(""), "city"
  end

  test "assigning a string over a sequence replaces the sequence" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rails
      name: "x"
    YAML

    document.root["tags"] = "none"

    assert_equal "tags: none\nname: \"x\"\n", document.to_s
  end

  test "assigning an integer over a map replaces the map" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: "Berlin"
      name: "x"
    YAML

    document.root["venue"] = 5432

    assert_equal "venue: 5432\nname: \"x\"\n", document.to_s
    assert_equal 5432, document.value_at("venue")
  end

  test "assigning a boolean over a sequence replaces the sequence" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
      name: "x"
    YAML

    document.root["tags"] = true

    assert_equal "tags: true\nname: \"x\"\n", document.to_s
    assert_equal true, document.value_at("tags")
  end

  test "assigning over a flow collection replaces it" do
    document = Yerba::Document.parse("venue: {city: Berlin}\ntags: [ruby]\n")

    document.root["venue"] = "none"
    document.root["tags"] = "none"

    assert_equal "venue: none\ntags: none\n", document.to_s
  end

  test "assigning over a nested map keeps the surrounding indentation" do
    document = Yerba::Document.parse(<<~YAML)
      a:
        b:
          c: 1
      d: 2
    YAML

    document["a"]["b"] = "none"

    assert_equal "a:\n  b: none\nd: 2\n", document.to_s
  end

  test "a comment on the key line survives the replacement" do
    document = Yerba::Document.parse(<<~YAML)
      venue: # why
        city: "Berlin"
      name: "x"
    YAML

    document.root["venue"] = "none"

    assert_equal "venue: none # why\nname: \"x\"\n", document.to_s
  end

  test "a comment following the collection survives the replacement" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: "Berlin"
      # note
      name: "x"
    YAML

    document.root["venue"] = "none"

    assert_equal "venue: none\n# note\nname: \"x\"\n", document.to_s
  end

  test "assigning a multi-key hash over a collection still works" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: "Berlin"
      name: "x"
    YAML

    document.root["venue"] = { "country" => "DE", "zip" => "AB" }

    assert_equal({ "country" => "DE", "zip" => "AB" }, document.value_at("venue"))
    assert_equal "x", document.value_at("name")
  end

  test "assigning an array over a collection still works" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: "Berlin"
      name: "x"
    YAML

    document.root["venue"] = ["a", "b"]

    assert_equal ["a", "b"], document.value_at("venue")
    assert_equal "x", document.value_at("name")
  end

  test "assigning a flow collection over a collection replaces it inline" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: "Berlin"
      name: "x"
    YAML

    document.root.set("venue", { "country" => "DE" }, style: :flow)

    assert_equal "venue: {country: DE}\nname: \"x\"\n", document.to_s
  end

  test "assigning over a scalar is unaffected" do
    document = Yerba::Document.parse(<<~YAML)
      name: "x"
      age: 5
    YAML

    document.root["name"] = "y"
    document.root["age"] = 6

    assert_equal "name: \"y\"\nage: 6\n", document.to_s
  end
end
