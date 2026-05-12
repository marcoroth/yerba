# frozen_string_literal: true

require "test_helper"

class MapTest < Minitest::Spec
  test "[] on map path returns Yerba::Map" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_instance_of Yerba::Map, document["database"]
  end

  test "map[] navigates to child" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_instance_of Yerba::Scalar, document["database"]["host"]
    assert_equal "localhost", document["database"]["host"].value
  end

  test "map[]= sets value" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML
    document["database"]["host"] = "0.0.0.0"

    assert_equal <<~YAML, document.to_s
      database:
        host: 0.0.0.0
    YAML
  end

  test "map.keys returns key names" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal ["host", "port"], document["database"].keys
  end

  test "map inspect shows keys and values" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal '#<Yerba::Map selector="database" {host: "localhost", port: 5432}>', document["database"].inspect
  end

  test "map.dig resolves nested value" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal "localhost", document["database"]["host"]
    assert_equal 5432, document["database"]["port"]
  end

  test "each yields key and bound node" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    pairs = document["database"].each.map { |key, node| [key, node.class.name] }

    assert_equal [["host", "Yerba::Scalar"], ["port", "Yerba::Scalar"]], pairs
  end

  test "mutating through map each" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"].each { |key, node| node.value = "changed" if key == "host" }

    assert_equal "changed", document.get("database.host")
  end

  test "map.insert adds new key at end" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"].insert("ssl", "true")

    expected = <<~YAML
      database:
        host: localhost
        port: 5432
        ssl: true
    YAML

    assert_equal expected, document.to_s
  end

  test "map.insert adds new key after specified key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"].insert("ssl", "true", after: "host")

    expected = <<~YAML
      database:
        host: localhost
        ssl: true
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.insert adds new key before specified key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"].insert("ssl", "true", before: "port")

    expected = <<~YAML
      database:
        host: localhost
        ssl: true
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.sort_keys orders keys" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        port: 5432
        host: localhost
    YAML
    document["database"].sort_keys(["host", "port"])

    expected = <<~YAML
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.delete removes a key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
        pool: 10
    YAML
    document["database"].delete("pool")

    expected = <<~YAML
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.key? returns true for existing key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert document["database"].key?("host")
    assert document["database"].key?("port")
  end

  test "map.key? returns false for missing key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    refute document["database"].key?("missing")
  end

  test "standalone map from hash" do
    map = Yerba::Map.new(name: "Alice", age: 30)

    assert_equal [:name, :age], map.keys
  end

  test "standalone map [] accesses values" do
    map = Yerba::Map.new(name: "Alice", age: 30)

    assert_equal "Alice", map[:name]
    assert_equal 30, map[:age]
  end

  test "standalone map []= sets values" do
    map = Yerba::Map.new(name: "Alice")
    map[:age] = 30

    assert_equal 30, map[:age]
  end

  test "standalone map to_yaml renders as YAML" do
    map = Yerba::Map.new(name: "Alice", port: 5432)
    yaml = map.to_yaml

    assert_includes yaml, "name: Alice"
    assert_includes yaml, "port: 5432"
  end

  test "standalone map to_yaml with scalar values respects quote_style" do
    map = Yerba::Map.new(
      name: Yerba::Scalar.new("Alice", quote_style: :double),
      active: Yerba::Scalar.new("true", quote_style: :double)
    )
    yaml = map.to_yaml

    assert_includes yaml, 'name: "Alice"'
    assert_includes yaml, 'active: "true"'
  end

  test "standalone map each iterates key-value pairs" do
    map = Yerba::Map.new(a: 1, b: 2)
    pairs = map.map { |k, v| [k, v] }

    assert_equal [[:a, 1], [:b, 2]], pairs
  end

  test "standalone map inspect" do
    map = Yerba::Map.new(name: "Alice")

    assert_equal '#<Yerba::Map {name: "Alice"}>', map.inspect
  end

  test "inserting map into sequence" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: "Ruby"
    YAML
    map = Yerba::Map.new(name: "Rust", year: 2015)
    document["items"] << map

    assert_includes document.to_s, '"Rust"'
    assert_includes document.to_s, "2015"
  end

  test "Map.new standalone with kwargs" do
    map = Yerba::Map.new(name: "Alice", age: 30)

    assert_equal "Alice", map[:name]
    assert_equal 30, map[:age]
    assert_nil map.selector
    assert_nil map.file_path
    refute map.connected?
  end

  test "Map.new standalone with hash" do
    map = Yerba::Map.new({ "name" => "Alice" })

    assert_equal "Alice", map["name"]
    refute map.connected?
  end

  test "Map.from creates map with metadata" do
    map = Yerba::Map.from(
      file_path: "/tmp/test.yml",
      selector: "[0]",
      line: 1
    )

    assert_equal "/tmp/test.yml", map.file_path
    assert_equal "[0]", map.selector
    assert_equal 1, map.line
    assert map.connected?
  end

  test "Map.from_document creates connected map" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    map = document["database"]

    assert_instance_of Yerba::Map, map
    assert_equal "database", map.selector
    assert_equal "localhost", map["host"].value
    assert map.connected?
  end

  test "Map.from_document has location" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    map = document["database"]

    assert map.location
    assert map.line
  end

  test "Map.from lazily loads document on read" do
    file = Tempfile.new(["test", ".yml"])
    file.write("- id: talk-1\n  title: First\n")
    file.close

    map = Yerba::Map.from(file_path: file.path, selector: "[0]")

    assert_nil map.instance_variable_get(:@document)
    assert_equal "talk-1", map["id"].value
    refute_nil map.document
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end

  test "Map.from lazily loads document on mutation" do
    file = Tempfile.new(["test", ".yml"])
    file.write("- id: talk-1\n  title: First\n")
    file.close

    map = Yerba::Map.from(file_path: file.path, selector: "[0]")
    map["title"] = "Updated"

    assert_equal <<~YAML, map.document.to_s
      - id: talk-1
        title: Updated
    YAML
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end

  test "Map.from shares document with scalars from same file" do
    file = Tempfile.new(["test", ".yml"])
    file.write("- id: talk-1\n  title: First\n")
    file.close

    map = Yerba::Map.from(file_path: file.path, selector: "[0]")
    scalar = Yerba::Scalar.from(file_path: file.path, selector: "[0].title")

    map["id"]
    scalar.value

    assert_same map.document, scalar.document
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end
end
