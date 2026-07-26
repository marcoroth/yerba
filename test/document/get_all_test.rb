# frozen_string_literal: true

require "test_helper"

class DocumentGetAllTest < Minitest::Spec
  SEQUENCE = <<~YAML
    - id: "a"
      title: "First"
      venue:
        city: "Berlin"
      talks:
        - id: "a1"
    - id: "b"
      title: "Second"
  YAML

  MAP = <<~YAML
    name: "Conf"
    year: 2024
    venue:
      city: "Berlin"
      country: "DE"
    tags:
      - "ruby"
      - "rails"
  YAML

  test "get_all resolves every node matching a selector" do
    document = Yerba.parse(SEQUENCE)

    assert_equal ["a", "b"], document.get_all("[].id").map(&:value)
    assert_equal ["First", "Second"], document.get_all("[].title").map(&:value)
  end

  test "get_all returns an empty array when nothing matches" do
    assert_empty Yerba.parse(SEQUENCE).get_all("[].missing")
  end

  test "get_all resolves nested selectors" do
    document = Yerba.parse(SEQUENCE)

    assert_equal ["a1"], document.get_all("[].talks[].id").map(&:value)
    assert_equal ["Berlin"], document.get_all("[].venue.city").map(&:value)
  end

  test "get_all returns the node type of each match" do
    document = Yerba.parse(SEQUENCE)

    assert_equal [Yerba::Map, Yerba::Map], document.get_all("[]").map(&:class)
    assert_equal [Yerba::Map], document.get_all("[].venue").map(&:class)
    assert_equal [Yerba::Sequence], document.get_all("[].talks").map(&:class)
    assert_equal [Yerba::Scalar, Yerba::Scalar], document.get_all("[].id").map(&:class)
  end

  test "get_all matches the nodes returned by indexing" do
    document = Yerba.parse(SEQUENCE)

    ["[]", "[].id", "[].title", "[].talks[].id"].each do |selector|
      expected = Array(document[selector])
      actual = document.get_all(selector)

      assert_equal expected.map(&:selector), actual.map(&:selector), selector
      assert_equal expected.map { |node| node.location&.start_line }, actual.map { |node| node.location&.start_line }, selector
    end
  end

  test "get_all reports selectors and locations" do
    nodes = Yerba.parse(SEQUENCE).get_all("[].id")

    assert_equal ["[0].id", "[1].id"], nodes.map(&:selector)
    assert_equal([1, 7], nodes.map { |node| node.location.start_line })
  end

  test "get_all returns nodes that are still connected to the document" do
    document = Yerba.parse(SEQUENCE)
    node = document.get_all("[]").first

    assert_equal "First", node["title"].value

    node["title"] = "Changed"

    assert_equal "Changed", document.value_at("[0].title")
  end

  test "get_all attaches the map key a value belongs to" do
    nodes = Yerba.parse(SEQUENCE).get_all("[].id")

    assert_equal(["id", "id"], nodes.map { |node| node.key.value })
    assert_equal([1, 7], nodes.map { |node| node.key.location.start_line })
  end

  test "get_all attaches keys for nested map values" do
    nodes = Yerba.parse(SEQUENCE).get_all("[].venue.city")

    assert_equal(["city"], nodes.map { |node| node.key.value })
  end

  test "get_all leaves sequence entries without a key" do
    document = Yerba.parse(SEQUENCE)

    assert_equal [nil, nil], document.get_all("[]").map(&:key)
    assert_equal [nil], document.get_all("[].talks[]").map(&:key)
  end

  test "get_all keys match the keys returned by indexing" do
    document = Yerba.parse(SEQUENCE)

    ["[]", "[].id", "[].venue", "[].venue.city", "[].talks[]", "[].talks[].id"].each do |selector|
      expected = Array(document[selector]).map { |node| [node.key&.value, node.key&.location&.start_line] }
      actual = document.get_all(selector).map { |node| [node.key&.value, node.key&.location&.start_line] }

      assert_equal expected, actual, selector
    end
  end

  test "* resolves every value of a map" do
    document = Yerba.parse(MAP)

    assert_equal ["Conf", 2024], document.get_all("*").first(2).map(&:value)
    assert_equal(["name", "year", "venue", "tags"], document.get_all("*").map { |node| node.key.value })
  end

  test "* resolves values of a nested map" do
    document = Yerba.parse(MAP)

    assert_equal ["Berlin", "DE"], document.get_all("venue.*").map(&:value)
    assert_equal ["venue.city", "venue.country"], document.get_all("venue.*").map(&:selector)
  end

  test "* keeps the node type of each value" do
    types = Yerba.parse(MAP).get_all("*").map(&:class)

    assert_equal [Yerba::Scalar, Yerba::Scalar, Yerba::Map, Yerba::Sequence], types
  end

  test "* resolves map values inside a sequence" do
    document = Yerba.parse(SEQUENCE)

    assert_equal(["id", "title", "venue", "talks"], document.get_all("[0].*").map { |node| node.key.value })
  end

  test "* descends into a sequence the same way a named key does" do
    document = Yerba.parse(SEQUENCE)

    assert_equal ["[0].talks[0].id"], document.get_all("[].talks.id").map(&:selector)
    assert_equal ["[0].talks[0].id"], document.get_all("[].talks.*").map(&:selector)
  end
end
