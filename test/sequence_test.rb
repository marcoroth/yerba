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

  test "find_by returns bound node for matching key/value" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        slug: alice
      - name: Bob
        slug: bob
    YAML

    result = document.root.find_by(name: "Bob")

    assert_instance_of Yerba::Map, result
    assert_equal "Bob", result["name"].value
    assert_equal "[1]", result.selector
  end

  test "find_by returns nil when no match" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
    YAML

    assert_nil document.root.find_by(name: "Missing")
  end

  test "find_by result is mutable" do
    document = Yerba::Document.parse(<<~YAML)
      - name: "Alice"
        github: ""
      - name: "Bob"
        github: ""
    YAML

    speaker = document.root.find_by(name: "Alice")
    speaker["github"] = "alice123"

    assert_equal <<~YAML, document.to_s
      - name: "Alice"
        github: "alice123"
      - name: "Bob"
        github: ""
    YAML
  end

  test "select_by returns all matching bound nodes" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        kind: speaker
      - name: Bob
        kind: organizer
      - name: Charlie
        kind: speaker
    YAML

    results = document.root.where(kind: "speaker")

    assert_equal 2, results.length
    assert_equal "Alice", results[0]["name"].value
    assert_equal "Charlie", results[1]["name"].value
  end

  test "find_by with multiple criteria narrows results" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        kind: speaker
      - name: Bob
        kind: speaker
      - name: Alice
        kind: organizer
    YAML

    result = document.root.find_by(name: "Alice", kind: "organizer")

    assert_equal "organizer", result["kind"].value
    assert_equal "[2]", result.selector
  end

  test "find_by with positional selector" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        kind: speaker
      - name: Bob
        kind: organizer
    YAML

    result = document.root.find_by("name", "Bob")

    assert_instance_of Yerba::Map, result
    assert_equal "Bob", result["name"].value
  end

  test "where returns all matching bound nodes" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        kind: speaker
      - name: Bob
        kind: organizer
      - name: Charlie
        kind: speaker
    YAML

    results = document.root.where(kind: "speaker")

    assert_equal 2, results.length
    assert_equal "Alice", results[0]["name"].value
    assert_equal "Charlie", results[1]["name"].value
  end

  test "index_of with kwargs for map sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
      - name: Bob
      - name: Charlie
    YAML

    assert_equal 1, document.root.index_of(name: "Bob")
    assert_nil document.root.index_of(name: "Missing")
  end

  test "pluck single field" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        slug: alice
      - name: Bob
        slug: bob
    YAML

    assert_equal ["Alice", "Bob"], document.root.pluck(:name)
  end

  test "pluck with missing fields returns nil" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        github: aalice
      - name: Bob
    YAML

    assert_equal ["aalice", nil], document.root.pluck(:github)
    assert_equal [["Alice", "aalice"], ["Bob", nil]], document.root.pluck(:name, :github)
  end

  test "pluck returns typed values" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        age: 30
        active: true
      - name: Bob
        age: 25
        active: false
    YAML

    assert_equal [30, 25], document.root.pluck(:age)
    assert_equal [true, false], document.root.pluck(:active)
    assert_equal [["Alice", 30, true], ["Bob", 25, false]], document.root.pluck(:name, :age, :active)
  end

  test "pluck multiple fields" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        slug: alice
        github: aalice
      - name: Bob
        slug: bob
        github: bbob
    YAML

    assert_equal [["Alice", "alice"], ["Bob", "bob"]], document.root.pluck(:name, :slug)
    assert_equal [["Alice", "alice", "aalice"], ["Bob", "bob", "bbob"]], document.root.pluck(:name, :slug, :github)
  end

  test "find_by with nested selector" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
          - name: Bob
      - id: talk-2
        speakers:
          - name: Charlie
    YAML

    result = document.root.find_by("speakers[].name", "Alice")

    assert_instance_of Yerba::Map, result
    assert_equal "talk-1", result["id"].value
  end

  test "find_by with nested selector finds correct entry" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
      - id: talk-2
        speakers:
          - name: Charlie
    YAML

    result = document.root.find_by("speakers[].name", "Charlie")

    assert_equal "talk-2", result["id"].value
  end

  test "find_by with nested hash syntax" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
          - name: Bob
      - id: talk-2
        speakers:
          - name: Charlie
    YAML

    result = document.root.find_by(speakers: { name: "Charlie" })

    assert_instance_of Yerba::Map, result
    assert_equal "talk-2", result["id"].value
  end

  test "where with nested hash syntax" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
      - id: talk-2
        speakers:
          - name: Alice
          - name: Bob
      - id: talk-3
        speakers:
          - name: Charlie
    YAML

    results = document.root.where(speakers: { name: "Alice" })

    assert_equal 2, results.length
    assert_equal "talk-1", results[0]["id"].value
    assert_equal "talk-2", results[1]["id"].value
  end

  test "find_by with nested hash and additional flat criteria" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        kind: keynote
        speakers:
          - name: Alice
      - id: talk-2
        kind: talk
        speakers:
          - name: Alice
    YAML

    result = document.root.find_by(kind: "talk", speakers: { name: "Alice" })

    assert_equal "talk-2", result["id"].value
  end

  test "find_by with deeply nested hash" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
            links:
              - url: https://alice.dev
      - id: talk-2
        speakers:
          - name: Bob
            links:
              - url: https://bob.dev
    YAML

    result = document.root.find_by(speakers: { links: { url: "https://bob.dev" } })

    assert_equal "talk-2", result["id"].value
  end

  test "sequence entry with nested sequence returns Map" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
    YAML

    assert_instance_of Yerba::Map, document[""][0]
    assert_instance_of Yerba::Sequence, document[""][0]["speakers"]
  end

  test "root sequence returns Sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - id: first
      - id: second
    YAML

    assert_instance_of Yerba::Sequence, document.root
  end

  test "index_of with kwargs on scalar sequence returns nil" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_nil document["tags"].index_of(name: "ruby")
  end

  test "index_of with scalar value on map sequence returns nil" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
      - name: Bob
    YAML

    assert_nil document.root.index_of("Alice")
  end

  test "index_of with scalar value for scalar sequence" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
        - go
    YAML

    assert_equal 0, document["tags"].index_of("ruby")
    assert_equal 2, document["tags"].index_of("go")
    assert_nil document["tags"].index_of("python")
  end
end
