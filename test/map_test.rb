# frozen_string_literal: true

require "test_helper"

class MapTest < Minitest::Spec
  test "[] on map path returns Yerba::Map" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert_instance_of Yerba::Map, document["database"]
  end

  test "map[] navigates to child" do
    document = Yerba::Document.parse("database:\n  host: localhost")

    assert_instance_of Yerba::Scalar, document["database"]["host"]
    assert_equal "localhost", document["database"]["host"].value
  end

  test "map[]= sets value" do
    document = Yerba::Document.parse("database:\n  host: localhost")
    document["database"]["host"] = "0.0.0.0"

    assert_equal "database:\n  host: 0.0.0.0", document.to_s
  end

  test "map.keys returns key names" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert_equal ["host", "port"], document["database"].keys
  end

  test "map inspect shows keys and values" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert_equal '#<Yerba::Map path="database" {host: "localhost", port: 5432}>', document["database"].inspect
  end

  test "map.dig resolves nested value" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert_equal "localhost", document["database"]["host"]
    assert_equal 5432, document["database"]["port"]
  end

  test "each yields key and bound node" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")
    pairs = document["database"].each.map { |key, node| [key, node.class.name] }

    assert_equal [["host", "Yerba::Scalar"], ["port", "Yerba::Scalar"]], pairs
  end

  test "mutating through map each" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")
    document["database"].each { |key, node| node.value = "changed" if key == "host" }

    assert_equal "changed", document.get("database.host")
  end

  test "map.insert adds new key at end" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")
    document["database"].insert("ssl", "true")

    expected = <<~YAML.chomp
      database:
        host: localhost
        port: 5432
        ssl: true
    YAML

    assert_equal expected, document.to_s
  end

  test "map.insert adds new key after specified key" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")
    document["database"].insert("ssl", "true", after: "host")

    expected = <<~YAML.chomp
      database:
        host: localhost
        ssl: true
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.insert adds new key before specified key" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")
    document["database"].insert("ssl", "true", before: "port")

    expected = <<~YAML.chomp
      database:
        host: localhost
        ssl: true
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.sort_keys orders keys" do
    document = Yerba::Document.parse("database:\n  port: 5432\n  host: localhost")
    document["database"].sort_keys(["host", "port"])

    expected = <<~YAML.chomp
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.delete removes a key" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432\n  pool: 10")
    document["database"].delete("pool")

    expected = <<~YAML.chomp
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal expected, document.to_s
  end

  test "map.key? returns true for existing key" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert document["database"].key?("host")
    assert document["database"].key?("port")
  end

  test "map.key? returns false for missing key" do
    document = Yerba::Document.parse("database:\n  host: localhost")

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
    document = Yerba::Document.parse("items:\n  - name: \"Ruby\"")
    map = Yerba::Map.new(name: "Rust", year: 2015)
    document["items"] << map

    assert_includes document.to_s, '"Rust"'
    assert_includes document.to_s, "2015"
  end
end
