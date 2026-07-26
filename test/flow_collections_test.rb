# frozen_string_literal: true

require "test_helper"

class FlowCollectionsTest < Minitest::Spec
  test "sequence each yields the entries of a flow sequence" do
    sequence = Yerba::Document.parse("tags: [ruby, rails]\n")["tags"]

    assert_equal 2, sequence.length
    assert_equal ["ruby", "rails"], sequence.each.to_a.map(&:value)
    assert_equal ["ruby", "rails"], sequence.map(&:value)
  end

  test "indexing a flow sequence returns a node" do
    sequence = Yerba::Document.parse("tags: [ruby, rails]\n")["tags"]

    assert_instance_of Yerba::Scalar, sequence[0]
    assert_equal "rails", sequence[1].value
    assert_nil sequence[2]
  end

  test "map each yields the entries of a flow map" do
    map = Yerba::Document.parse("venue: {city: Berlin, country: DE}\n")["venue"]

    assert_equal ["city", "country"], map.keys
    assert_equal([["city", "Berlin"], ["country", "DE"]], map.each.to_a.map { |key, value| [key, value.value] })
  end

  test "indexing a flow map returns a node" do
    map = Yerba::Document.parse("venue: {city: Berlin, country: DE}\n")["venue"]

    assert_equal "Berlin", map["city"].value
    assert_nil map["missing"]
  end

  test "flow nodes report their own selector and location" do
    document = Yerba::Document.parse("tags: [ruby, rails]\n")
    nodes = document.get_all("tags[]")

    assert_equal ["tags[0]", "tags[1]"], nodes.map(&:selector)
    assert_equal([1, 1], nodes.map { |node| node.location.start_line })
  end

  test "a flow map value carries the key it belongs to" do
    nodes = Yerba::Document.parse("venue: {city: Berlin, country: DE}\n").get_all("venue.*")

    assert_equal(["city", "country"], nodes.map { |node| node.key.value })
  end

  test "values read through a flow sequence can be written back" do
    document = Yerba::Document.parse("tags: [ruby, rails]\n")

    document["tags"].each { |tag| tag.value = tag.value.upcase }

    assert_equal "tags: [RUBY, RAILS]\n", document.to_s
  end

  test "values read through a flow map can be written back" do
    document = Yerba::Document.parse("venue: {city: Berlin, country: DE}\n")

    document["venue"]["city"] = "Hamburg"

    assert_equal "venue: {city: Hamburg, country: DE}\n", document.to_s
  end

  test "iteration counts agree with length for flow and block alike" do
    ["tags: [a, b, c]\n", "tags:\n  - a\n  - b\n  - c\n"].each do |source|
      sequence = Yerba::Document.parse(source)["tags"]

      assert_equal sequence.length, sequence.each.to_a.length, source
      assert_equal ["a", "b", "c"], sequence.map(&:value), source
    end
  end

  test "deleting an entry of a flow sequence is refused" do
    document = Yerba::Document.parse("tags: [ruby, rails]\n")

    error = assert_raises(Yerba::Error) { document.delete("tags[0]") }

    assert_includes error.message, "flow collections are not writable"
    assert_equal "tags: [ruby, rails]\n", document.to_s
  end

  test "deleting a key of a flow map is refused" do
    document = Yerba::Document.parse("venue: {city: Berlin, country: DE}\n")

    assert_raises(Yerba::Error) { document.delete("venue.city") }
    assert_equal "venue: {city: Berlin, country: DE}\n", document.to_s
  end

  test "deleting the whole entry holding a flow collection still works" do
    document = Yerba::Document.parse("tags: [ruby]\nname: x\n")

    document.delete("tags")

    assert_equal "name: x\n", document.to_s
  end

  test "adding a key to an occupied flow map is refused" do
    document = Yerba::Document.parse("venue: {city: Berlin}\n")

    assert_raises(Yerba::Error) { document.insert("venue.zip", "12345") }
    assert_equal "venue: {city: Berlin}\n", document.to_s
  end

  test "adding a key to an empty flow map still converts it to block style" do
    document = Yerba::Document.parse("metadata: {}\n")

    document.root["metadata"]["source"] = "youtube"

    assert_equal "metadata:\n  source: youtube\n", document.to_s
  end

  test "converting to block style makes a flow collection writable" do
    document = Yerba::Document.parse("tags: [ruby, rails]\n")

    document["tags"].collection_style = :block
    document.delete("tags[0]")

    assert_equal "rails", document.value_at("tags[0]")
  end
end
