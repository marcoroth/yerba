# frozen_string_literal: true

require "test_helper"

class SequenceTest < Minitest::Spec
  test "[] on sequence path returns Yerba::Sequence" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust")

    assert_instance_of Yerba::Sequence, document["tags"]
  end

  test "sequence.each iterates items" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust\n  - go")

    assert_equal ["ruby", "rust", "go"], document["tags"].map(&:to_s)
  end

  test "sequence.length returns count" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust\n  - go")

    assert_equal 3, document["tags"].length
  end

  test "sequence.first and .last" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust\n  - go")

    assert_equal "ruby", document["tags"].first
    assert_equal "go", document["tags"].last
  end

  test "sequence << inserts scalar" do
    document = Yerba::Document.parse("tags:\n  - ruby")
    document["tags"] << "rust"

    assert_includes document.to_s, "- rust"
  end

  test "sequence << inserts hash" do
    document = Yerba::Document.parse("items:\n  - name: \"Ruby\"")
    document["items"] << { name: "Rust" }

    assert_includes document.to_s, '"Rust"'
  end

  test "sequence << inserts Yerba::Scalar with quote_style" do
    document = Yerba::Document.parse("tags:\n  - ruby")
    document["tags"] << Yerba::Scalar.new("true", quote_style: :double)

    assert_includes document.to_s, '- "true"'
  end

  test "sequence << raises on non-sequence" do
    document = Yerba::Document.parse("name: Alice")

    assert_raises(NoMethodError) do
      document["name"] << "value"
    end
  end

  test "sequence inspect shows items" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust")

    assert_equal '#<Yerba::Sequence path="tags" ["ruby", "rust"]>', document["tags"].inspect
  end

  test "sequence.include? checks membership" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust")

    assert document["tags"].include?("rust")
    refute document["tags"].include?("go")
  end

  test "sequence[index] returns correct type for map entries" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n    year: 1995")

    assert_instance_of Yerba::Map, document["items"][0]
    assert_instance_of Yerba::Scalar, document["items"][0]["name"]
  end

  test "assignment through sequence index" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n    year: 1995\n  - name: Rust\n    year: 2015")
    document["items"][0]["name"] = "Go"

    assert_equal "Go", document.dig("items", 0, "name")
  end

  test "each yields bound nodes for sequence of maps" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")
    names = document["items"].map { |item| item["name"].value }

    assert_equal ["Ruby", "Rust"], names
  end

  test "first returns bound node for sequence of maps" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")

    assert_instance_of Yerba::Map, document["items"].first
    assert_equal "Ruby", document["items"].first["name"].value
  end

  test "mutating through sequence navigation" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")
    document["items"].first["name"] = "Go"

    assert_equal "Go", document.get("items[0].name")
  end

  test "standalone sequence from array" do
    seq = Yerba::Sequence.new(["ruby", "rust", "go"])

    assert_equal 3, seq.length
  end

  test "standalone sequence each iterates items" do
    seq = Yerba::Sequence.new(["ruby", "rust"])

    assert_equal ["ruby", "rust"], seq.to_a
  end

  test "standalone sequence << appends items" do
    seq = Yerba::Sequence.new(["ruby"])
    seq << "rust"

    assert_equal ["ruby", "rust"], seq.to_a
  end

  test "standalone sequence [] accesses by index" do
    seq = Yerba::Sequence.new(["ruby", "rust", "go"])

    assert_equal "rust", seq[1]
  end

  test "standalone sequence to_yaml renders items" do
    seq = Yerba::Sequence.new(["ruby", "rust"])

    assert_equal "- ruby\n- rust", seq.to_yaml
  end

  test "standalone sequence inspect" do
    seq = Yerba::Sequence.new(["ruby", "rust"])

    assert_equal '#<Yerba::Sequence ["ruby", "rust"]>', seq.inspect
  end
end
