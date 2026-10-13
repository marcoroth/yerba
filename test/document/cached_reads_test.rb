# frozen_string_literal: true

require "test_helper"

class CachedReadsTest < Minitest::Spec
  SOURCE = <<~YAML
    - name: Ruby
      slug: ruby
      tags:
        - lang
        - dynamic
      year: 1995
      stable: true
      license:
    - name: Rust
      slug: rust
  YAML

  test "value_at reads concrete selectors" do
    document = Yerba::Document.parse(SOURCE)

    assert_equal "Ruby", document.value_at("[0].name")
    assert_equal ["lang", "dynamic"], document.value_at("[0].tags")
    assert_equal "dynamic", document.value_at("[0].tags[1]")
    assert_equal 1995, document.value_at("[0].year")
    assert_equal true, document.value_at("[0].stable")
    assert_nil document.value_at("[0].license")
    assert_equal "rust", document.value_at("[1].slug")
  end

  test "value_at handles wildcards and missing paths" do
    document = Yerba::Document.parse(SOURCE)

    assert_equal ["Ruby", "Rust"], document.value_at("[].name")
    assert_nil document.value_at("[0].missing")
    assert_nil document.value_at("[5].name")
  end

  test "value_at indexes into maps like the native lookup" do
    document = Yerba::Document.parse("name: Ruby\ntags:\n  - lang\n  - dynamic\n")

    assert_equal "lang", document.value_at("[0]")
    assert_equal "dynamic", document.value_at("[1]")
    assert_nil Yerba::Document.parse("name: Ruby\n").value_at("[0]")
  end

  test "value_at does not hand out references into the cached values" do
    document = Yerba::Document.parse(SOURCE)

    document.value_at("[0].name") << "!"
    document.value_at("[0].tags") << "mutated"

    assert_equal "Ruby", document.value_at("[0].name")
    assert_equal ["lang", "dynamic"], document.value_at("[0].tags")
  end

  test "map and sequence reads see edits made through the document" do
    document = Yerba::Document.parse(SOURCE)
    first = document["[0]"]

    assert_equal "ruby", first.value_at("slug")

    first["slug"] = "ruby-lang"
    document["[0].tags"] << "typed"

    assert_equal "ruby-lang", first.value_at("slug")
    assert_equal "typed", document["[0].tags"].value_at(2)
    assert_equal 3, document["[0].tags"].length
  end

  test "length_at counts sequence items" do
    document = Yerba::Document.parse(SOURCE)

    assert_equal 2, document.length_at("")
    assert_equal 2, document.length_at("[0].tags")
    assert_equal 0, document.length_at("[1].tags")
    assert_equal 0, document.length_at("[0].name")
  end

  test "sequence length matches each" do
    document = Yerba::Document.parse(SOURCE)
    tags = document["[0].tags"]

    assert_equal 2, tags.length
    assert_equal tags.length, tags.each.to_a.length
    assert_equal ["lang", "dynamic"], tags.map(&:value)
  end

  test "each yields nodes whose location and key match native lookups" do
    document = Yerba::Document.parse(SOURCE)

    document["[0]"].each do |name, child|
      native = document["[0].#{name}"]

      assert_equal native.class, child.class
      assert_equal native.location, child.location
      assert_equal native.key.value, child.key.value
      assert_equal native.key.location, child.key.location
    end
  end

  test "each falls back for entries without a value" do
    document = Yerba::Document.parse(SOURCE)
    names = document["[0]"].map { |name, _| name }

    assert_equal ["name", "slug", "tags", "year", "stable", "license"], names
  end

  test "each yields empty flow collections" do
    document = Yerba::Document.parse("item:\n  name: Ruby\n  talks: []\n  meta: {}\n")
    yielded = {}

    document["item"].each { |name, child| yielded[name] = child.class }

    assert_equal({ "name" => Yerba::Scalar, "talks" => Yerba::Sequence, "meta" => Yerba::Map }, yielded)
  end

  test "wildcard selectors match native results" do
    document = Yerba::Document.parse(SOURCE)
    nodes = document["[].name"]

    assert_equal ["[0].name", "[1].name"], nodes.map(&:selector)
    assert_equal ["Ruby", "Rust"], nodes.map(&:value)
    assert_equal [1, 9], nodes.map(&:line)
    assert_equal ["lang", "dynamic"], document["[0].tags[]"].map(&:value)
  end

  test "wildcard selectors fall back when an item lacks the key" do
    document = Yerba::Document.parse(SOURCE)

    assert_equal ["[0].tags"], document["[].tags"].map(&:selector)
  end

  test "mutating a yielded scalar value does not leak into later reads" do
    document = Yerba::Document.parse(SOURCE)

    document["[].name"].first.value << "!"

    assert_equal "Ruby", document["[].name"].first.value
    assert_equal "Ruby", document.value_at("[0].name")
  end

  test "children of a node read before an edit reflect the edit" do
    document = Yerba::Document.parse(SOURCE)
    tags = document["[0].tags"]

    document["[0].tags"] << "typed"

    assert_equal ["lang", "dynamic", "typed"], tags.children.map(&:value)
  end

  test "lookups hand out fresh nodes" do
    document = Yerba::Document.parse(SOURCE)

    refute_same document["[0]"], document["[0]"]
    refute_same document["[0].tags"].children.first, document["[0].tags"].children.first

    document["[0].tags"].children << "mutated"

    assert_equal 2, document["[0].tags"].children.length
  end

  test "entries without a value are yielded but skipped by wildcards" do
    document = Yerba::Document.parse(SOURCE)
    license = document["[0]"].children["license"]

    assert_instance_of Yerba::Scalar, license
    assert_nil license.value
    assert_equal 8, license.line
    assert_equal "license", license.key.value
    assert_equal license.location, document["[0].license"].location
    assert_empty document["[].license"]
  end
end
