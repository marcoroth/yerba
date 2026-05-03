# frozen_string_literal: true

require "test_helper"

class SequenceTest < Minitest::Spec
  test "[] on sequence path returns Yerba::Sequence" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_instance_of Yerba::Sequence, document["tags"]
  end

  test "sequence.each iterates items" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
        - go
    YAML

    assert_equal ["ruby", "rust", "go"], document["tags"].map(&:to_s)
  end

  test "sequence.length returns count" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
        - go
    YAML

    assert_equal 3, document["tags"].length
  end

  test "sequence.first and .last" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
        - go
    YAML

    assert_equal "ruby", document["tags"].first
    assert_equal "go", document["tags"].last
  end

  test "sequence << inserts scalar" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML
    document["tags"] << "rust"

    assert_includes document.to_s, "- rust"
  end

  test "sequence << inserts hash matching double quote style" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: "Ruby"
          year: 1995
    YAML

    document["items"] << { name: "Rust", year: 2015 }

    assert_equal <<~YAML, document.to_s
      items:
        - name: "Ruby"
          year: 1995
        - name: "Rust"
          year: 2015
    YAML
  end

  test "sequence << inserts hash matching plain style" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
          year: 1995
    YAML

    document["items"] << { name: "Rust", year: 2015 }

    assert_equal <<~YAML, document.to_s
      items:
        - name: Ruby
          year: 1995
        - name: Rust
          year: 2015
    YAML
  end

  test "sequence << inserts scalar matching existing quote style" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - "ruby"
    YAML
    document["tags"] << "rust"

    expected = <<~YAML
      tags:
        - "ruby"
        - "rust"
    YAML

    assert_equal expected, document.to_s
  end

  test "sequence << inserts scalar as plain when existing items are plain" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML
    document["tags"] << "rust"

    expected = <<~YAML
      tags:
        - ruby
        - rust
    YAML

    assert_equal expected, document.to_s
  end

  test "sequence << inserts Yerba::Scalar with quote_style" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML
    document["tags"] << Yerba::Scalar.new("true", quote_style: :double)

    assert_includes document.to_s, '- "true"'
  end

  test "sequence << raises on non-sequence" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(NoMethodError) do
      document["name"] << "value"
    end
  end

  test "sequence inspect shows items" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_equal '#<Yerba::Sequence selector="tags" ["ruby", "rust"]>', document["tags"].inspect
  end

  test "sequence.include? checks membership" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert document["tags"].include?("rust")
    refute document["tags"].include?("go")
  end

  test "sequence[index] returns correct type for map entries" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
          year: 1995
    YAML

    assert_instance_of Yerba::Map, document["items"][0]
    assert_instance_of Yerba::Scalar, document["items"][0]["name"]
  end

  test "assignment through sequence index" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
          year: 1995
        - name: Rust
          year: 2015
    YAML
    document["items"][0]["name"] = "Go"

    assert_equal "Go", document.dig("items", 0, "name")
  end

  test "each yields bound nodes for sequence of maps" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    names = document["items"].map { |item| item["name"].value }

    assert_equal ["Ruby", "Rust"], names
  end

  test "first returns bound node for sequence of maps" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML

    assert_instance_of Yerba::Map, document["items"].first
    assert_equal "Ruby", document["items"].first["name"].value
  end

  test "mutating through sequence navigation" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    document["items"].first["name"] = "Go"

    assert_equal "Go", document.get("items[0].name")
  end

  test "remove deletes item by value" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
        - go
    YAML
    document["tags"].remove("rust")

    refute_includes document.to_s, "rust"
    assert_includes document.to_s, "ruby"
    assert_includes document.to_s, "go"
  end

  test "sort orders scalar items" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - rust
        - go
        - ruby
    YAML
    document["tags"].sort

    lines = document.to_s.lines.map(&:strip).select { |line| line.start_with?("- ") }

    assert_equal ["- go", "- ruby", "- rust"], lines
  end

  test "sort with by: orders by field" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Rust
          year: 2015
        - name: Go
          year: 2009
    YAML
    document["items"].sort(by: "name")

    assert_equal "Go", document.get("items[0].name")
    assert_equal "Rust", document.get("items[1].name")
  end

  test "delete_at removes item by index" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
        - name: Go
    YAML
    document["items"].delete_at(1)

    assert_equal "Ruby", document.get("items[0].name")
    assert_equal "Go", document.get("items[1].name")
  end

  test "delete_if removes items matching block" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
          year: 1995
        - name: Rust
          year: 2015
        - name: Go
          year: 2009
    YAML
    document["items"].delete_if { |item| item["year"].value > 2000 }

    assert_equal 1, document["items"].length
    assert_equal "Ruby", document.get("items[0].name")
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
