# frozen_string_literal: true

require "test_helper"

class DocumentTest < Minitest::Spec
  test "Yerba.parse parses YAML content" do
    document = Yerba.parse("name: Alice")

    assert_equal "Alice", document.get("name")
    assert_instance_of Yerba::Document, document
  end

  test "Yerba.parse_file parses a file" do
    require "tempfile"

    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\n")
    file.close

    document = Yerba.parse_file(file.path)

    assert_equal "Alice", document.get("name")
    assert_equal file.path, document.path
  ensure
    file&.unlink
  end

  test "Document.parse parses YAML content" do
    document = Yerba::Document.parse("name: Alice")

    assert_equal "Alice", document.get("name")
  end

  test "Document.parse returns string representation" do
    document = Yerba::Document.parse("key: value")

    assert_equal "key: value", document.to_s
  end

  test "get returns string for plain string scalar" do
    document = Yerba::Document.parse("name: Alice")
    result = document.get("name")

    assert_equal "Alice", result
    assert_instance_of String, result
  end

  test "get returns integer for plain integer scalar" do
    document = Yerba::Document.parse("port: 5432")
    result = document.get("port")

    assert_equal 5432, result
    assert_instance_of Integer, result
  end

  test "get returns float for plain float scalar" do
    document = Yerba::Document.parse("ratio: 3.14")
    result = document.get("ratio")

    assert_in_delta 3.14, result
    assert_instance_of Float, result
  end

  test "get returns true for plain boolean true" do
    document = Yerba::Document.parse("ssl: true")

    assert_equal true, document.get("ssl")
  end

  test "get returns false for plain boolean false" do
    document = Yerba::Document.parse("debug: false")

    assert_equal false, document.get("debug")
  end

  test "get returns nil for plain null" do
    document = Yerba::Document.parse("timeout: null")

    assert_nil document.get("timeout")
  end

  test "get returns string for quoted boolean" do
    document = Yerba::Document.parse('flag: "true"')
    result = document.get("flag")

    assert_equal "true", result
    assert_instance_of String, result
  end

  test "get returns string for quoted number" do
    document = Yerba::Document.parse('version: "3.2"')
    result = document.get("version")

    assert_equal "3.2", result
    assert_instance_of String, result
  end

  test "get returns nil for missing path" do
    document = Yerba::Document.parse("name: Alice")

    assert_nil document.get("missing")
  end

  test "get returns array for wildcard path" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")
    result = document.get("items[].name")

    assert_equal ["Ruby", "Rust"], result
    assert_instance_of Array, result
  end

  test "get returns typed array elements" do
    document = Yerba::Document.parse("items:\n  - count: 1\n  - count: 2")
    result = document.get("items[].count")

    assert_equal [1, 2], result
    assert_instance_of Integer, result.first
  end

  test "exists? returns true for existing path" do
    document = Yerba::Document.parse("name: Alice")

    assert document.exists?("name")
  end

  test "exists? returns false for missing path" do
    document = Yerba::Document.parse("name: Alice")

    refute document.exists?("missing")
  end

  test "set preserves quotes for string value" do
    document = Yerba::Document.parse('name: "hello"')
    document.set("name", "world")

    assert_includes document.to_s, '"world"'
  end

  test "set forces plain scalar for integer" do
    document = Yerba::Document.parse('port: "5432"')
    document.set("port", 5433)

    assert_includes document.to_s, "port: 5433"
    refute_includes document.to_s, '"5433"'
  end

  test "set forces plain scalar for boolean" do
    document = Yerba::Document.parse('ssl: "false"')
    document.set("ssl", true)

    assert_includes document.to_s, "ssl: true"
    refute_includes document.to_s, '"true"'
  end

  test "set writes null for nil" do
    document = Yerba::Document.parse("timeout: 30")
    document.set("timeout", nil)

    assert_includes document.to_s, "timeout: null"
  end

  test "set returns self for chaining" do
    document = Yerba::Document.parse("a: 1\nb: 2")
    result = document.set("a", 10)

    assert_same document, result
  end

  test "insert appends to sequence" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust")
    document.insert("tags", "go")

    assert_includes document.to_s, "- go"
  end

  test "insert_object inserts hash as YAML mapping" do
    document = Yerba::Document.parse("items:\n  - name: \"Ruby\"\n    year: 1995")

    document.insert_object("items", { name: "Rust", year: 2015 })
    output = document.to_s

    assert_includes output, '"Rust"'
    assert_includes output, "2015"
  end

  test "delete removes a key" do
    document = Yerba::Document.parse("name: Alice\nage: 30")

    document.delete("age")

    refute_includes document.to_s, "age"
    assert_includes document.to_s, "name: Alice"
  end

  test "sort orders sequence items" do
    document = Yerba::Document.parse("tags:\n  - rust\n  - go\n  - ruby")
    document.sort("tags")

    lines = document.to_s.lines.map(&:strip)
    tag_lines = lines.select { |line| line.start_with?("- ") }

    assert_equal ["- go", "- ruby", "- rust"], tag_lines
  end

  test "sort_keys orders map keys" do
    document = Yerba::Document.parse("port: 5432\nhost: localhost\nname: mydb")
    document.sort_keys("", ["host", "name", "port"])
    lines = document.to_s.lines.map(&:chomp)

    assert_match(/^host:/, lines[0])
    assert_match(/^name:/, lines[1])
    assert_match(/^port:/, lines[2])
  end

  test "save! writes content to file" do
    require "tempfile"

    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\n")
    file.close

    document = Yerba::Document.new(file.path)
    document.set("name", "New")
    document.save!

    assert_equal "name: New\n", File.read(file.path)
  ensure
    file&.unlink
  end

  test "dig resolves nested string value" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert_equal "localhost", document.dig("database", "host")
    assert_equal 5432, document.dig("database", "port")
  end

  test "dig with integer index into sequence" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")

    assert_equal "Ruby", document.dig("items", 0, "name")
    assert_equal "Rust", document.dig("items", 1, "name")
  end

  test "dig returns nil for missing path" do
    document = Yerba::Document.parse("database:\n  host: localhost")

    assert_nil document.dig("database", "missing")
  end

  test "root returns Map for map document" do
    document = Yerba::Document.parse("name: Alice\nport: 5432")

    assert_instance_of Yerba::Map, document.root
  end

  test "root returns Sequence for sequence document" do
    document = Yerba::Document.parse("- name: Hello\n- name: World")

    assert_instance_of Yerba::Sequence, document.root
  end

  test "map? returns true for map document" do
    document = Yerba::Document.parse("name: Alice")

    assert document.map?
    refute document.sequence?
  end

  test "sequence? returns true for sequence document" do
    document = Yerba::Document.parse("- ruby\n- rust")

    assert document.sequence?
    refute document.map?
  end

  test "at_path returns Scalar for scalar path" do
    document = Yerba::Document.parse("database:\n  host: localhost")

    assert_instance_of Yerba::Scalar, document.at_path("database.host")
    assert_equal "localhost", document.at_path("database.host").value
  end

  test "at_path returns Map for map path" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert_instance_of Yerba::Map, document.at_path("database")
  end

  test "at_path returns Sequence for sequence path" do
    document = Yerba::Document.parse("tags:\n  - ruby\n  - rust")

    assert_instance_of Yerba::Sequence, document.at_path("tags")
  end

  test "at_path with index returns typed node" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")

    assert_instance_of Yerba::Map, document.at_path("items[0]")
    assert_instance_of Yerba::Scalar, document.at_path("items[0].name")
    assert_equal "Ruby", document.at_path("items[0].name").value
  end

  test "at_path with wildcard returns array of typed nodes" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")
    nodes = document.at_path("items[].name")

    assert_instance_of Array, nodes
    assert_equal 2, nodes.length
    assert_instance_of Yerba::Scalar, nodes.first
    assert_equal "Ruby", nodes.first.value
    assert_equal "Rust", nodes.last.value
  end

  test "at_path with wildcard allows mutation" do
    document = Yerba::Document.parse("items:\n  - name: Ruby\n  - name: Rust")
    document.at_path("items[].name").each { |node| node.value = "Go" }

    assert_equal "Go", document.get("items[0].name")
    assert_equal "Go", document.get("items[1].name")
  end

  test "to_h returns parsed Ruby object for map document" do
    document = Yerba::Document.parse("name: Alice\nport: 5432\nssl: true")

    assert_equal({ "name" => "Alice", "port" => 5432, "ssl" => true }, document.to_h)
  end

  test "to_a returns parsed Ruby object for sequence document" do
    document = Yerba::Document.parse("- name: Hello\n- name: World")

    assert_equal [{ "name" => "Hello" }, { "name" => "World" }], document.to_a
  end

  test "[] returns nil for non-existent path" do
    document = Yerba::Document.parse("name: Alice")

    assert_nil document["missing"]
  end
end
