# frozen_string_literal: true

require "test_helper"

class DocumentTest < Minitest::Spec
  test "Yerba.parse parses YAML content" do
    document = Yerba.parse(<<~YAML)
      name: Alice
    YAML

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
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal "Alice", document.get("name")
  end

  test "Document.parse returns string representation" do
    document = Yerba::Document.parse(<<~YAML)
      key: value
    YAML

    assert_equal <<~YAML, document.to_s
      key: value
    YAML
  end

  test "get returns string for plain string scalar" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    result = document.get("name")

    assert_equal "Alice", result
    assert_instance_of String, result
  end

  test "get returns integer for plain integer scalar" do
    document = Yerba::Document.parse(<<~YAML)
      port: 5432
    YAML
    result = document.get("port")

    assert_equal 5432, result
    assert_instance_of Integer, result
  end

  test "get returns float for plain float scalar" do
    document = Yerba::Document.parse(<<~YAML)
      ratio: 3.14
    YAML
    result = document.get("ratio")

    assert_in_delta 3.14, result
    assert_instance_of Float, result
  end

  test "get returns true for plain boolean true" do
    document = Yerba::Document.parse(<<~YAML)
      ssl: true
    YAML

    assert_equal true, document.get("ssl")
  end

  test "get returns false for plain boolean false" do
    document = Yerba::Document.parse(<<~YAML)
      debug: false
    YAML

    assert_equal false, document.get("debug")
  end

  test "get returns nil for plain null" do
    document = Yerba::Document.parse(<<~YAML)
      timeout: null
    YAML

    assert_nil document.get("timeout")
  end

  test "get returns string for quoted boolean" do
    document = Yerba::Document.parse(<<~YAML)
      flag: "true"
    YAML
    result = document.get("flag")

    assert_equal "true", result
    assert_instance_of String, result
  end

  test "get returns string for quoted number" do
    document = Yerba::Document.parse(<<~YAML)
      version: "3.2"
    YAML
    result = document.get("version")

    assert_equal "3.2", result
    assert_instance_of String, result
  end

  test "get returns nil for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_nil document.get("missing")
  end

  test "get returns array for wildcard path" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    result = document.get("items[].name")

    assert_equal ["Ruby", "Rust"], result
    assert_instance_of Array, result
  end

  test "get returns typed array elements" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - count: 1
        - count: 2
    YAML
    result = document.get("items[].count")

    assert_equal [1, 2], result
    assert_instance_of Integer, result.first
  end

  test "exists? returns true for existing path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert document.exists?("name")
  end

  test "exists? returns false for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    refute document.exists?("missing")
  end

  test "set preserves quotes for string value" do
    document = Yerba::Document.parse(<<~YAML)
      name: "hello"
    YAML
    document.set("name", "world")

    assert_includes document.to_s, '"world"'
  end

  test "set forces plain scalar for integer" do
    document = Yerba::Document.parse(<<~YAML)
      port: "5432"
    YAML
    document.set("port", 5433)

    assert_includes document.to_s, "port: 5433"
    refute_includes document.to_s, '"5433"'
  end

  test "set forces plain scalar for boolean" do
    document = Yerba::Document.parse(<<~YAML)
      ssl: "false"
    YAML
    document.set("ssl", true)

    assert_includes document.to_s, "ssl: true"
    refute_includes document.to_s, '"true"'
  end

  test "set writes null for nil" do
    document = Yerba::Document.parse(<<~YAML)
      timeout: 30
    YAML
    document.set("timeout", nil)

    assert_includes document.to_s, "timeout: null"
  end

  test "set returns self for chaining" do
    document = Yerba::Document.parse(<<~YAML)
      a: 1
      b: 2
    YAML
    result = document.set("a", 10)

    assert_same document, result
  end

  test "insert appends to sequence" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML
    document.insert("tags", "go")

    assert_includes document.to_s, "- go"
  end

  test "insert_object inserts hash as YAML mapping" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: "Ruby"
          year: 1995
    YAML

    document.insert_object("items", { name: "Rust", year: 2015 })
    output = document.to_s

    assert_includes output, '"Rust"'
    assert_includes output, "2015"
  end

  test "delete removes a key" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      age: 30
    YAML

    document.delete("age")

    refute_includes document.to_s, "age"
    assert_includes document.to_s, "name: Alice"
  end

  test "sort_keys orders map keys" do
    document = Yerba::Document.parse(<<~YAML)
      port: 5432
      host: localhost
      name: mydb
    YAML
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
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal "localhost", document.dig("database", "host")
    assert_equal 5432, document.dig("database", "port")
  end

  test "dig with integer index into sequence" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML

    assert_equal "Ruby", document.dig("items", 0, "name")
    assert_equal "Rust", document.dig("items", 1, "name")
  end

  test "dig returns nil for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_nil document.dig("database", "missing")
  end

  test "root returns Map for map document" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    assert_instance_of Yerba::Map, document.root
  end

  test "root returns Sequence for sequence document" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Hello
      - name: World
    YAML

    assert_instance_of Yerba::Sequence, document.root
  end

  test "map? returns true for map document" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert document.map?
    refute document.sequence?
  end

  test "sequence? returns true for sequence document" do
    document = Yerba::Document.parse(<<~YAML)
      - ruby
      - rust
    YAML

    assert document.sequence?
    refute document.map?
  end

  test "at_path returns Scalar for scalar path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_instance_of Yerba::Scalar, document.at_path("database.host")
    assert_equal "localhost", document.at_path("database.host").value
  end

  test "at_path returns Map for map path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_instance_of Yerba::Map, document.at_path("database")
  end

  test "at_path returns Sequence for sequence path" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_instance_of Yerba::Sequence, document.at_path("tags")
  end

  test "at_path with index returns typed node" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML

    assert_instance_of Yerba::Map, document.at_path("items[0]")
    assert_instance_of Yerba::Scalar, document.at_path("items[0].name")
    assert_equal "Ruby", document.at_path("items[0].name").value
  end

  test "at_path with wildcard returns array of typed nodes" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    nodes = document.at_path("items[].name")

    assert_instance_of Array, nodes
    assert_equal 2, nodes.length
    assert_instance_of Yerba::Scalar, nodes.first
    assert_equal "Ruby", nodes.first.value
    assert_equal "Rust", nodes.last.value
  end

  test "at_path with wildcard allows mutation" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    document.at_path("items[].name").each { |node| node.value = "Go" }

    assert_equal "Go", document.get("items[0].name")
    assert_equal "Go", document.get("items[1].name")
  end

  test "get raises on invalid path with trailing dot" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document.get("name.") }
  end

  test "get raises on invalid path with double dot" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document.get("name..x") }
  end

  test "get raises on invalid path with leading dot" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document.get(".name") }
  end

  test "get raises on invalid path with unclosed bracket" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document.get("[") }
  end

  test "set raises on invalid path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document.set("name.", "Bob") }
  end

  test "delete raises on invalid path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document.delete("name.") }
  end

  test "[] raises on invalid path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document["name."] }
  end

  test "to_h returns parsed Ruby object for map document" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
      ssl: true
    YAML

    assert_equal({ "name" => "Alice", "port" => 5432, "ssl" => true }, document.to_h)
  end

  test "to_a returns parsed Ruby object for sequence document" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Hello
      - name: World
    YAML

    assert_equal [{ "name" => "Hello" }, { "name" => "World" }], document.to_a
  end

  test "get_value returns hash for map path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal({ "host" => "localhost", "port" => 5432 }, document.get_value("database"))
  end

  test "get_value returns array for sequence path" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_equal ["ruby", "rust"], document.get_value("tags")
  end

  test "get_value returns scalar for scalar path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    assert_equal "Alice", document.get_value("name")
    assert_equal 5432, document.get_value("port")
  end

  test "get_value returns nil for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_nil document.get_value("missing")
  end

  test "get_value returns full document for empty path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    assert_equal({ "name" => "Alice", "port" => 5432 }, document.get_value(""))
  end

  test "get_values returns array of values for wildcard path" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
          year: 1995
        - name: Rust
          year: 2015
    YAML

    assert_equal [{ "name" => "Ruby", "year" => 1995 }, { "name" => "Rust", "year" => 2015 }], document.get_values("items[]")
  end

  test "get_values returns scalar values for scalar wildcard" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_equal ["ruby", "rust"], document.get_values("tags[]")
  end

  test "get_values returns empty array for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal [], document.get_values("missing[]")
  end

  test "find returns items from root sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - id: abc
        title: Hello
      - id: def
        title: World
    YAML
    results = document.find("[]")

    assert_equal 2, results.length
    assert_equal "abc", results[0]["id"]
  end

  test "find with condition filters items" do
    document = Yerba::Document.parse(<<~YAML)
      - id: abc
        kind: talk
      - id: def
        kind: keynote
    YAML
    results = document.find("[]", condition: '.kind == "keynote"')

    assert_equal 1, results.length
    assert_equal "def", results[0]["id"]
  end

  test "find with select returns only specified fields" do
    document = Yerba::Document.parse(<<~YAML)
      - id: abc
        title: Hello
        year: 2020
      - id: def
        title: World
        year: 2021
    YAML
    results = document.find("[]", select: "id,title")

    assert_equal 2, results.length
    assert_equal({ "id" => "abc", "title" => "Hello" }, results[0])
    assert_equal({ "id" => "def", "title" => "World" }, results[1])
  end

  test "find returns items from nested sequence" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - id: abc
          title: Hello
        - id: def
          title: World
    YAML
    results = document.find("items[]")

    assert_equal 2, results.length
    assert_equal "abc", results[0]["id"]
    assert_equal "def", results[1]["id"]
  end

  test "find with condition on nested sequence" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - id: abc
          kind: talk
        - id: def
          kind: keynote
        - id: ghi
          kind: talk
    YAML
    results = document.find("items[]", condition: '.kind == "keynote"')

    assert_equal 1, results.length
    assert_equal "def", results[0]["id"]
  end

  test "find with select on nested sequence" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - id: abc
          title: Hello
          year: 2020
        - id: def
          title: World
          year: 2021
    YAML
    results = document.find("items[]", select: "id,title")

    assert_equal 2, results.length
    assert_equal({ "id" => "abc", "title" => "Hello" }, results[0])
    assert_equal({ "id" => "def", "title" => "World" }, results[1])
  end

  test "[] returns nil for non-existent path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_nil document["missing"]
  end

  test "set with all: true updates all matching nodes" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Hello
        description: "some text"
      - title: World
        description: "other text"
    YAML

    document.set("[].description", "", all: true)

    assert_equal <<~YAML, document.to_s
      - title: Hello
        description: ""
      - title: World
        description: ""
    YAML
  end

  test "document.find_by delegates to root sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        role: admin
      - name: Bob
        role: user
    YAML

    result = document.find_by(name: "Bob")

    assert_instance_of Yerba::Map, result
    assert_equal "user", result["role"].value
  end

  test "document.where delegates to root sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        role: admin
      - name: Bob
        role: user
      - name: Charlie
        role: admin
    YAML

    results = document.where(role: "admin")

    assert_equal 2, results.length
    assert_equal "Alice", results[0]["name"].value
    assert_equal "Charlie", results[1]["name"].value
  end

  test "document.pluck delegates to root sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
      - name: Bob
      - name: Charlie
    YAML

    names = document.pluck(:name)

    assert_equal ["Alice", "Bob", "Charlie"], names
  end

  test "document << appends to root sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
      - name: Bob
    YAML

    document << { name: "Charlie" }

    assert_equal 3, document.root.length
    assert_equal "Charlie", document.find_by(name: "Charlie")["name"].value
  end
end
