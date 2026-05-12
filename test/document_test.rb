# frozen_string_literal: true

require "test_helper"
require "tempfile"
require "fileutils"

class DocumentTest < Minitest::Spec
  test "Yerba.parse parses YAML content" do
    document = Yerba.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal "Alice", document.value_at("name")
    assert_instance_of Yerba::Document, document
  end

  test "Yerba.parse_file parses a file" do
    require "tempfile"

    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\n")
    file.close

    document = Yerba.parse_file(file.path)

    assert_equal "Alice", document.value_at("name")
    assert_equal file.path, document.path
  ensure
    file&.unlink
  end

  test "Document.parse parses YAML content" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal "Alice", document.value_at("name")
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
    result = document.value_at("name")

    assert_equal "Alice", result
    assert_instance_of String, result
  end

  test "get returns integer for plain integer scalar" do
    document = Yerba::Document.parse(<<~YAML)
      port: 5432
    YAML
    result = document.value_at("port")

    assert_equal 5432, result
    assert_instance_of Integer, result
  end

  test "get returns float for plain float scalar" do
    document = Yerba::Document.parse(<<~YAML)
      ratio: 3.14
    YAML
    result = document.value_at("ratio")

    assert_in_delta 3.14, result
    assert_instance_of Float, result
  end

  test "get returns true for plain boolean true" do
    document = Yerba::Document.parse(<<~YAML)
      ssl: true
    YAML

    assert_equal true, document.value_at("ssl")
  end

  test "get returns false for plain boolean false" do
    document = Yerba::Document.parse(<<~YAML)
      debug: false
    YAML

    assert_equal false, document.value_at("debug")
  end

  test "get returns nil for plain null" do
    document = Yerba::Document.parse(<<~YAML)
      timeout: null
    YAML

    assert_nil document.value_at("timeout")
  end

  test "get returns string for quoted boolean" do
    document = Yerba::Document.parse(<<~YAML)
      flag: "true"
    YAML
    result = document.value_at("flag")

    assert_equal "true", result
    assert_instance_of String, result
  end

  test "get returns string for quoted number" do
    document = Yerba::Document.parse(<<~YAML)
      version: "3.2"
    YAML
    result = document.value_at("version")

    assert_equal "3.2", result
    assert_instance_of String, result
  end

  test "get returns nil for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_nil document.value_at("missing")
  end

  test "get returns array for wildcard path" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    result = document.value_at("items[].name")

    assert_equal ["Ruby", "Rust"], result
    assert_instance_of Array, result
  end

  test "get returns typed array elements" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - count: 1
        - count: 2
    YAML
    result = document.value_at("items[].count")

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

    assert_instance_of Yerba::Scalar, document.dig("database", "host")
    assert_equal "localhost", document.dig("database", "host").value
    assert_equal 5432, document.dig("database", "port").value
  end

  test "dig with integer index into sequence" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML

    assert_instance_of Yerba::Scalar, document.dig("items", 0, "name")
    assert_equal "Ruby", document.dig("items", 0, "name").value
    assert_equal "Rust", document.dig("items", 1, "name").value
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

  test "[] with wildcard returns array of typed nodes" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    nodes = document["items[].name"]

    assert_instance_of Array, nodes
    assert_equal 2, nodes.length
    assert_instance_of Yerba::Scalar, nodes.first
    assert_equal "Ruby", nodes.first.value
    assert_equal "Rust", nodes.last.value
  end

  test "[] with wildcard allows mutation" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML
    document["items[].name"].each { |node| node.value = "Go" }

    assert_equal "Go", document.value_at("items[0].name")
    assert_equal "Go", document.value_at("items[1].name")
  end

  test "[] with wildcard returns Maps" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
          year: 1995
        - name: Rust
          year: 2015
    YAML
    nodes = document["items[]"]

    assert_instance_of Array, nodes
    assert_equal 2, nodes.length
    assert_instance_of Yerba::Map, nodes.first
  end

  test "[] with root wildcard returns array" do
    document = Yerba::Document.parse(<<~YAML)
      - name: Ruby
      - name: Rust
    YAML
    nodes = document["[].name"]

    assert_instance_of Array, nodes
    assert_equal 2, nodes.length
    assert_equal "Ruby", nodes.first.value
  end

  test "[]= updates existing key" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML
    document["name"] = "Bob"

    assert_equal "Bob", document.value_at("name")
  end

  test "[]= inserts new key" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document["age"] = 30

    assert_equal 30, document.value_at("age")
    assert_equal <<~YAML, document.to_s
      name: Alice
      age: 30
    YAML
  end

  test "[]= preserves string type" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document["count"] = "42"

    assert_instance_of String, document.value_at("count")
  end

  test "[] raises on invalid path with trailing dot" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document["name."] }
  end

  test "[] raises on invalid path with double dot" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document["name..x"] }
  end

  test "[] raises on invalid path with leading dot" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document[".name"] }
  end

  test "[] raises on invalid path with unclosed bracket" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::PathValidationError) { document["["] }
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

    assert_equal({ "host" => "localhost", "port" => 5432 }, document.value_at("database"))
  end

  test "get_value returns array for sequence path" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_equal ["ruby", "rust"], document.value_at("tags")
  end

  test "get_value returns scalar for scalar path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    assert_equal "Alice", document.value_at("name")
    assert_equal 5432, document.value_at("port")
  end

  test "get_value returns nil for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_nil document.value_at("missing")
  end

  test "get_value returns full document for empty path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    assert_equal({ "name" => "Alice", "port" => 5432 }, document.value_at(""))
  end

  test "value_at returns array of values for wildcard path" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
          year: 1995
        - name: Rust
          year: 2015
    YAML

    assert_equal [{ "name" => "Ruby", "year" => 1995 }, { "name" => "Rust", "year" => 2015 }], document.value_at("items[]")
  end

  test "value_at returns scalar values for scalar wildcard" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_equal ["ruby", "rust"], document.value_at("tags[]")
  end

  test "value_at returns nil for missing wildcard path" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_nil document.value_at("missing[]")
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

  test "document.apply reorders keys from Yerbafile" do
    yerbafile = Tempfile.new(["Yerbafile", ".yml"])
    yerbafile.write(<<~YAML)
      rules:
        - files: "**/*.yml"
          pipeline:
            - sort_keys:
                path: "[]"
                order:
                  - name
                  - slug
                  - github
    YAML
    yerbafile.close

    document = Yerba::Document.parse(<<~YAML)
      - github: aalice
        name: Alice
        slug: alice
    YAML

    document.apply(yerbafile.path)

    assert_equal <<~YAML, document.to_s
      - name: Alice
        slug: alice
        github: aalice
    YAML
  ensure
    yerbafile&.unlink
  end

  test "document.apply with quote_style enforces style" do
    yerbafile = Tempfile.new(["Yerbafile", ".yml"])
    yerbafile.write(<<~YAML)
      rules:
        - files: "**/*.yml"
          pipeline:
            - quote_style:
                key_style: plain
                value_style: double
    YAML

    yerbafile.close

    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    document.apply(yerbafile.path)

    assert_equal <<~YAML, document.to_s
      name: "Alice"
    YAML
  ensure
    yerbafile&.unlink
  end

  test "document.save! with apply: true applies rules before saving" do
    dir = Dir.mktmpdir
    yerbafile_path = File.join(dir, "Yerbafile")
    File.write(yerbafile_path, <<~YAML)
      rules:
        - files: "**/*.yml"
          pipeline:
            - sort_keys:
                path: "[]"
                order:
                  - name
                  - slug
    YAML

    file_path = File.join(dir, "test.yml")
    File.write(file_path, "- slug: alice\n  name: Alice\n")

    Dir.chdir(dir) do
      document = Yerba.parse_file(file_path)
      document.save!(apply: true)
    end

    assert_equal <<~YAML, File.read(file_path)
      - name: Alice
        slug: alice
    YAML
  ensure
    FileUtils.rm_rf(dir)
  end

  test "document.selector returns empty string" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal "", document.selector
  end

  test "scalar.selector returns the selector path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_equal "database.host", document["database"]["host"].selector
    assert_equal "database.host", document["database.host"].selector
  end

  test "map.selector returns the selector path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_equal "database", document["database"].selector
  end

  test "sequence.selector returns the selector path" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    assert_equal "tags", document["tags"].selector
  end

  test "selector on indexed sequence item" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        title: First
      - id: talk-2
        title: Second
    YAML

    assert_equal "[0]", document["[0]"].selector
    assert_equal "[0].id", document["[0]"]["id"].selector
    assert_equal "[1].title", document["[1]"]["title"].selector
  end

  test "selector on nested structures" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
    YAML

    assert_equal "[0].speakers", document[0]["speakers"].selector
    assert_equal "[0].speakers[0]", document[0]["speakers"][0].selector
    assert_equal "[0].speakers[0].name", document[0]["speakers"][0]["name"].selector
  end

  test "integer index access on document" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - name: Alice
    YAML

    assert_equal "[0]", document[0].selector
    assert_equal "[0].id", document[0]["id"].selector
    assert_equal "[0].speakers", document[0]["speakers"].selector
    assert_equal "[0].speakers[0]", document[0]["speakers"][0].selector
    assert_equal "[0].speakers[0].name", document[0]["speakers"][0]["name"].selector
    assert_equal "Alice", document[0]["speakers"][0]["name"].value
  end

  test "document.save! without apply does not reorder keys" do
    dir = Dir.mktmpdir
    yerbafile_path = File.join(dir, "Yerbafile")

    File.write(yerbafile_path, <<~YAML)
      rules:
        - files: "**/*.yml"
          pipeline:
            - sort_keys:
                path: "[]"
                order:
                  - name
                  - slug
    YAML

    file_path = File.join(dir, "test.yml")
    File.write(file_path, "- slug: alice\n  name: Alice\n")

    Dir.chdir(dir) do
      document = Yerba.parse_file(file_path)
      document.save!
    end

    assert_equal <<~YAML, File.read(file_path)
      - slug: alice
        name: Alice
    YAML
  ensure
    FileUtils.rm_rf(dir)
  end

  test "document.find returns all entries" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        kind: keynote
      - title: Workshop
        kind: talk
    YAML

    results = document.find("[]")

    assert_equal 2, results.length
    assert_equal "Keynote", results[0]["title"]
    assert_equal "Workshop", results[1]["title"]
  end

  test "document.find with condition filters entries" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        kind: keynote
      - title: Workshop
        kind: talk
      - title: Panel
        kind: keynote
    YAML

    results = document.find("[]", condition: '.kind == "keynote"')

    assert_equal 2, results.length
    assert_equal "Keynote", results[0]["title"]
    assert_equal "Panel", results[1]["title"]
  end

  test "document.find with select returns only specified fields" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        kind: keynote
        year: 2024
      - title: Workshop
        kind: talk
        year: 2025
    YAML

    results = document.find("[]", select: "title,kind")

    assert_equal 2, results.length
    assert_equal "Keynote", results[0]["title"]
    assert_equal "keynote", results[0]["kind"]
    refute results[0].key?("year")
  end

  test "document.find with condition and select" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        kind: keynote
        speakers:
          - Alice
      - title: Workshop
        kind: talk
        speakers:
          - Bob
      - title: Panel
        kind: keynote
        speakers:
          - Charlie
    YAML

    results = document.find("[]", condition: '.kind == "keynote"', select: "title,speakers")

    assert_equal 2, results.length
    assert_equal "Keynote", results[0]["title"]
    assert_equal ["Alice"], results[0]["speakers"]
    refute results[0].key?("kind")
    assert_equal "Panel", results[1]["title"]
    assert_equal ["Charlie"], results[1]["speakers"]
  end

  test "document.find with contains condition" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        slides_url: "https://speakerdeck.com/alice"
      - title: Workshop
        slides_url: ""
      - title: Panel
        slides_url: "https://example.com/slides"
    YAML

    results = document.find("[]", condition: ".slides_url contains speakerdeck", select: "title,slides_url")

    assert_equal 1, results.length
    assert_equal "Keynote", results[0]["title"]
    assert_equal "https://speakerdeck.com/alice", results[0]["slides_url"]
  end

  test "document.find with not_contains condition" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        kind: keynote
      - title: Workshop
        kind: talk
      - title: Panel
        kind: keynote
    YAML

    results = document.find("[]", condition: ".kind not_contains keynote")

    assert_equal 1, results.length
    assert_equal "Workshop", results[0]["title"]
  end

  test "document.find with != condition" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        kind: keynote
      - title: Workshop
        kind: talk
    YAML

    results = document.find("[]", condition: '.kind != "keynote"')

    assert_equal 1, results.length
    assert_equal "Workshop", results[0]["title"]
  end

  test "document.find returns empty array when no matches" do
    document = Yerba::Document.parse(<<~YAML)
      - title: Keynote
        kind: keynote
    YAML

    results = document.find("[]", condition: '.kind == "talk"')

    assert_equal [], results
  end

  test "document.valid? returns true for valid document" do
    schema = {
      type: "object",
      properties: { name: { type: "string" } },
      required: ["name"],
    }

    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert document.valid?(schema)
  end

  test "document.valid? returns false for invalid document" do
    schema = {
      type: "object",
      properties: { name: { type: "string" } },
      required: ["name"],
    }

    document = Yerba::Document.parse(<<~YAML)
      slug: alice
    YAML

    refute document.valid?(schema)
  end

  test "document.validate returns errors with details" do
    schema = {
      type: "object",
      properties: {
        name: { type: "string" },
        slug: { type: "string" },
      },
      required: ["name", "slug"],
    }

    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        slug: alice
      - name: Bob
    YAML

    errors = document.validate(schema, selector: "[]")

    assert_equal 1, errors.length
    assert_equal "Bob", errors[0]["item_label"]
    assert_includes errors[0]["message"], "slug"
    assert_equal "/1", errors[0]["path"]
    assert_equal 3, errors[0]["line"]
  end

  test "document.validate accepts JSON string schema" do
    schema_json = '{"type":"object","properties":{"name":{"type":"string"}},"required":["name"]}'

    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal [], document.validate(schema_json)
  end

  test "document.validate with selector validates each item" do
    schema = {
      type: "object",
      properties: { name: { type: "string" } },
      required: ["name"],
      additionalProperties: false,
    }

    document = Yerba::Document.parse(<<~YAML)
      - name: Alice
        extra: bad
    YAML

    errors = document.validate(schema, selector: "[]")

    assert_equal 1, errors.length
    assert_includes errors[0]["message"], "extra"
  end

  test "document.valid? with empty array returns true" do
    schema = {
      type: "object",
      properties: { name: { type: "string" } },
      required: ["name"],
    }

    document = Yerba::Document.parse(<<~YAML)
      ---
      []
    YAML

    assert document.valid?(schema, selector: "[]")
  end

  test "get returns clean text for block scalar" do
    document = Yerba::Document.parse(<<~YAML)
      description: |-
        Hello World
    YAML

    assert_equal "Hello World", document.value_at("description")
  end

  test "get returns multiline block scalar with newlines" do
    document = Yerba::Document.parse(<<~YAML)
      description: |-
        First line.
        Second line.
        Third line.
    YAML

    assert_equal "First line.\nSecond line.\nThird line.", document.value_at("description")
  end

  test "get block scalar has no leading newline or whitespace" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        description: |-
          Some text here.
    YAML

    value = document.value_at("[0].description")

    refute value.start_with?("\n")
    refute value.start_with?(" ")
    assert_equal "Some text here.", value
  end

  test "get block scalar nested in sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        description: |-
          First paragraph.
          Second paragraph.
      - id: talk-2
        description: |-
          Another description.
    YAML

    assert_equal "First paragraph.\nSecond paragraph.", document.value_at("[0].description")
    assert_equal "Another description.", document.value_at("[1].description")
  end

  test "get_value returns clean string for block scalar" do
    document = Yerba::Document.parse(<<~YAML)
      description: |-
        Hello World
    YAML

    assert_equal "Hello World", document.value_at("description")
  end

  test "get_value returns multiline block scalar with newlines" do
    document = Yerba::Document.parse(<<~YAML)
      description: |-
        First line.
        Second line.
    YAML

    assert_equal "First line.\nSecond line.", document.value_at("description")
  end

  test "scalar.value returns clean text for block scalar" do
    document = Yerba::Document.parse(<<~YAML)
      description: |-
        Hello World
    YAML

    assert_equal "Hello World", document["description"].value
  end

  test "scalar.value returns multiline block scalar with newlines" do
    document = Yerba::Document.parse(<<~YAML)
      description: |-
        First paragraph.
        Second paragraph.
    YAML

    assert_equal "First paragraph.\nSecond paragraph.", document["description"].value
  end

  test "fetch returns Scalar for valid selector" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    scalar = document.fetch("name")

    assert_instance_of Yerba::Scalar, scalar
    assert_equal "Alice", scalar.value
  end

  test "fetch returns Map for valid map selector" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    map = document.fetch("database")

    assert_instance_of Yerba::Map, map
  end

  test "fetch raises SelectorNotFoundError for invalid selector on sequence" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        title: First
    YAML

    error = assert_raises(Yerba::SelectorNotFoundError) { document.fetch("title") }

    assert_includes error.message, "selector \"title\" is not valid"
    assert_includes error.message, "[].title"
  end

  test "fetch raises SelectorNotFoundError for invalid selector on map" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    error = assert_raises(Yerba::SelectorNotFoundError) { document.fetch("nonexistent") }

    assert_includes error.message, "selector \"nonexistent\" is not valid"
  end

  test "fetch suggests similar selectors" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        speakers:
          - Alice
    YAML

    error = assert_raises(Yerba::SelectorNotFoundError) { document.fetch("speakers") }

    assert_includes error.message, "Did you mean"
    assert_includes error.message, "[].speakers"
  end

  test "fetch shows available selectors when no close match" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    error = assert_raises(Yerba::SelectorNotFoundError) { document.fetch("zzzzz") }

    assert_includes error.message, "Available selectors"
    assert_includes error.message, "name"
  end

  test "[] returns nil for invalid selector without raising" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        title: First
    YAML

    assert_nil document["nonexistent"]
    assert_nil document["foo.bar.baz"]
  end

  test "get returns nil for invalid selector without raising" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        title: First
    YAML

    assert_nil document.value_at("nonexistent")
    assert_nil document.value_at("foo.bar.baz")
  end

  test "valid_selector? returns true for valid selectors" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        title: First
        speakers:
          - Alice
    YAML

    assert document.valid_selector?("[].id")
    assert document.valid_selector?("[].title")
    assert document.valid_selector?("[].speakers")
    assert document.valid_selector?("[].speakers[]")
    assert document.valid_selector?("[]")
    assert document.valid_selector?("[0].title")
  end

  test "valid_selector? returns false for invalid selectors" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        title: First
    YAML

    refute document.valid_selector?("title")
    refute document.valid_selector?("speakers")
    refute document.valid_selector?("nonexistent")
    refute document.valid_selector?("database.host")
  end

  test "validate_selector! does not raise for valid selectors" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    document.validate_selector!("name")
    document.validate_selector!("port")
  end

  test "validate_selector! raises for invalid selectors" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::SelectorNotFoundError) { document.validate_selector!("missing") }
  end

  test "fetch with bracket index works" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
      - id: talk-2
    YAML

    scalar = document.fetch("[1].id")

    assert_equal "talk-2", scalar.value
  end

  test "fetch returns nil for valid selector with missing value" do
    document = Yerba::Document.parse(<<~YAML)
      - id: talk-1
        title: First
      - id: talk-2
    YAML

    assert_nil document.fetch("[1].title")
  end
end
