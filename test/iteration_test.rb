# frozen_string_literal: true

require "test_helper"

class IterationTest < Minitest::Spec
  test "sequence each visits an entry that has no value" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - a
        -
        - c
    YAML

    sequence = document["items"]

    assert_equal 3, sequence.length
    assert_equal sequence.length, sequence.each.to_a.length
    assert_equal ["a", "c"], sequence.each.to_a.grep(Yerba::Scalar).map(&:value)
  end

  test "sequence each visits every entry of a flow sequence" do
    document = Yerba::Document.parse("tags: [ruby, rails]\n")
    sequence = document["tags"]

    assert_equal 2, sequence.length
    assert_equal sequence.length, sequence.each.to_a.length
  end

  test "sequence each yields as many entries as indexing does" do
    sources = [
      "items:\n  - a\n  - b\n",
      "items:\n  - a\n  -\n  - c\n",
      "items: [a, b]\n",
      "items: []\n"
    ]

    sources.each do |source|
      sequence = Yerba::Document.parse(source)["items"]
      indexed = sequence.length.times.map { |index| sequence[index] }

      assert_equal indexed.length, sequence.each.to_a.length, source
      assert_equal indexed.map(&:class), sequence.each.to_a.map(&:class), source
    end
  end

  test "sequence each returns self" do
    sequence = Yerba::Document.parse("items:\n  - a\n")["items"]

    assert_same(sequence, sequence.each { |item| item })
  end

  test "map each visits a key whose value is empty" do
    document = Yerba::Document.parse(<<~YAML)
      a: 1
      b:
      c: 3
    YAML

    pairs = document.root.each.to_a

    assert_equal ["a", "b", "c"], pairs.map(&:first)
    assert_equal([1, nil, 3], pairs.map { |_key, value| value.value })
  end

  test "map each pairs every value with its own key" do
    document = Yerba::Document.parse(<<~YAML)
      name: Conf
      venue:
        city: Berlin
      tags:
        - ruby
    YAML

    document.root.each do |key, value|
      assert_equal document[key].location.start_line, value.location.start_line, key
    end
  end

  test "map each visits every key of a flow map" do
    document = Yerba::Document.parse("venue: { city: Berlin, country: DE }\n")

    assert_equal ["city", "country"], document["venue"].each.to_a.map(&:first)
  end

  test "map each yields as many pairs as there are keys" do
    sources = [
      "a: 1\nb: 2\n",
      "a: 1\nb:\nc: 3\n",
      "a: { b: 1 }\n",
      "a:\n  b: 1\n"
    ]

    sources.each do |source|
      map = Yerba::Document.parse(source).root

      assert_equal map.keys.length, map.each.to_a.length, source
      assert_equal map.keys, map.each.to_a.map(&:first), source
    end
  end

  test "map each returns self" do
    map = Yerba::Document.parse("a: 1\n").root

    assert_same(map, map.each { |key, value| [key, value] })
  end
end
