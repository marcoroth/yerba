# frozen_string_literal: true

require "test_helper"

class NodesTest < Minitest::Spec
  SOURCE = <<~YAML
    - name: Ruby
      tags:
        - lang
      license:
    - name: Rust
  YAML

  test "nodes returns the root with every descendant" do
    root = Yerba::Document.parse(SOURCE).nodes
    first = root.children[0]
    tag = first.children["tags"].children[0]

    assert_instance_of Yerba::Sequence, root
    assert_equal "", root.selector
    assert_instance_of Yerba::Map, first
    assert_equal ["name", "tags", "license"], first.children.keys
    assert_equal "[0].tags[0]", tag.selector
    assert_equal "lang", tag.value
  end

  test "nodes have the same location and key as a lookup" do
    document = Yerba::Document.parse(SOURCE)
    node = document.nodes.children[1].children["name"]
    lookup = document["[1].name"]

    assert_equal lookup.location, node.location
    assert_equal lookup.key.value, node.key.value
    assert_equal lookup.key.location, node.key.location
  end

  test "nodes is cached until the document changes" do
    document = Yerba::Document.parse(SOURCE)
    root = document.nodes

    assert_same root, document.nodes

    document["[1].name"].value = "Crystal"

    refute_same root, document.nodes
    assert_equal "Crystal", document.nodes.children[1].children["name"].value
  end

  test "children returns copies of the cached nodes" do
    tags = Yerba::Document.parse(SOURCE).nodes.children[0].children["tags"]

    refute_same tags.children.first, tags.children.first

    tags.children << "mutated"
    tags.children.first.value << "!"

    assert_equal ["lang"], tags.children.map(&:value)
  end

  test "entries without a value are part of the tree" do
    document = Yerba::Document.parse(SOURCE)
    license = document.nodes.children[0].children["license"]

    assert_instance_of Yerba::Scalar, license
    assert_nil license.value
    assert_equal 4, license.line
    assert_equal "license", license.key.value
    assert_equal document["[0].license"].location, license.location
  end
end
