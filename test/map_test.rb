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

  test "map.keys returns the root map's own keys" do
    document = Yerba::Document.parse(<<~YAML)
      name: Conf
      year: 2024
      database:
        host: localhost
    YAML

    assert_equal ["name", "year", "database"], document.root.keys
  end

  test "map.keys is not confused by a nested sequence of maps" do
    document = Yerba::Document.parse(<<~YAML)
      id: "aloha"
      name: "Aloha"
      channels:
        - id: "UC123"
          handle: "@confreaks"
    YAML

    assert_equal ["id", "name", "channels"], document.root.keys
  end

  test "map.each yields every key and its value node" do
    document = Yerba::Document.parse(<<~YAML)
      name: Conf
      venue:
        city: Berlin
      tags:
        - ruby
    YAML

    pairs = document.root.each.to_a

    assert_equal ["name", "venue", "tags"], pairs.map(&:first)
    assert_equal([Yerba::Scalar, Yerba::Map, Yerba::Sequence], pairs.map { |_key, value| value.class })
    assert_equal "Berlin", pairs[1].last["city"].value
  end

  test "map.each yields values matching the ones indexing returns" do
    document = Yerba::Document.parse(<<~YAML)
      name: Conf
      venue:
        city: Berlin
    YAML

    map = document.root

    assert_equal(map.keys.map { |key| map[key].location&.start_line },
                 map.each.to_a.map { |_key, value| value.location&.start_line })
  end

  test "map.each yields values that can still be written to" do
    document = Yerba::Document.parse(<<~YAML)
      venue:
        city: Berlin
    YAML

    _key, venue = document.root.each.to_a.first
    venue["city"] = "Hamburg"

    assert_equal "Hamburg", document.value_at("venue.city")
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

    assert_equal "changed", document.value_at("database.host")
  end

  test "[]= updates existing key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"]["host"] = "0.0.0.0"

    assert_equal "0.0.0.0", document.value_at("database.host")
  end

  test "[]= inserts new string key when not present" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"]["name"] = "mydb"

    assert_equal "mydb", document.value_at("database.name")
    assert_includes document.to_s, "name: mydb"
  end

  test "[]= inserts new key on root map" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["age"] = 30

    assert_equal 30, document.value_at("age")

    assert_equal <<~YAML, document.to_s
      name: Alice
      age: 30
    YAML
  end

  test "[]= preserves type distinction between string and integer" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["age_string"] = "30"
    document.root["age_int"] = 30

    assert_instance_of String, document.value_at("age_string")
    assert_instance_of Integer, document.value_at("age_int")
    assert_equal "30", document.value_at("age_string")
    assert_equal 30, document.value_at("age_int")
  end

  test "[]= inserts with non-string types" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["age"] = 30
    document.root["active"] = true
    document.root["score"] = 9.5

    assert_equal <<~YAML, document.to_s
      name: Alice
      age: 30
      active: true
      score: 9.5
    YAML
  end

  test "map.insert adds new key at end" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"].insert("ssl", true)

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
    document["database"].insert("ssl", true, after: "host")

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
    document["database"].insert("ssl", true, before: "port")

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

  test "map.delete removes an array entry" do
    document = Yerba::Document.parse(<<~YAML)
      - name: "Entry 1"
      - name: "Entry 2"
    YAML
    document["[1]"].delete

    expected = <<~YAML
      - name: "Entry 1"
    YAML

    assert_equal expected, document.to_s
  end

  test "map.delete on only value replaces sequence with []" do
    document = Yerba::Document.parse(<<~YAML)
      tier:
        sponsors:
          - name: "Typesense"
    YAML
    document["tier.sponsors[0]"].delete

    assert_equal [], document["tier.sponsors"].value
  end

  test "map.delete removes a map" do
    document = Yerba::Document.parse(<<~YAML)
      speaker:
        name: "Rachael"
        slug: "rachael-wright-munn"
        github: "chaelcodes"
        aliases:
          name: "Chael"
          slug: "chaelcodes"
    YAML
    document["speaker.aliases"].delete

    expected = <<~YAML
      speaker:
        name: "Rachael"
        slug: "rachael-wright-munn"
        github: "chaelcodes"
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

  test "fetch returns node for existing key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    result = document["database"].fetch("host")

    assert_instance_of Yerba::Scalar, result
    assert_equal "localhost", result.value
  end

  test "fetch raises SelectorNotFoundError for missing key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_raises(Yerba::SelectorNotFoundError) { document["database"].fetch("missing") }
  end

  test "fetch suggests similar keys" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    error = assert_raises(Yerba::SelectorNotFoundError) { document["database"].fetch("hots") }

    assert_includes error.message, "host"
  end

  test "dig returns node for nested path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        settings:
          pool: 5
    YAML

    result = document["database"].dig("settings", "pool")

    assert_instance_of Yerba::Scalar, result
    assert_equal 5, result.value
  end

  test "dig returns nil for missing path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_nil document["database"].dig("missing", "key")
  end

  test "dig returns Map for intermediate path" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        settings:
          pool: 5
    YAML

    result = document["database"].dig("settings")

    assert_instance_of Yerba::Map, result
  end

  test "value_at returns plain value" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal "localhost", document["database"].value_at("host")
    assert_equal 5432, document["database"].value_at("port")
  end

  test "value_at returns nil for missing key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_nil document["database"].value_at("missing")
  end

  test "value_at returns hash for nested map" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        settings:
          pool: 5
          timeout: 30
    YAML

    assert_equal({ "pool" => 5, "timeout" => 30 }, document["database"].value_at("settings"))
  end

  test "[]= sets empty array" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["tags"] = []

    assert_equal <<~YAML, document.to_s
      name: Alice
      tags: []
    YAML
  end

  test "[]= sets array with values (block by default)" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["tags"] = ["ruby", "rails"]

    assert_equal <<~YAML, document.to_s
      name: Alice
      tags:
        - ruby
        - rails
    YAML
  end

  test "[]= sets array with values (flow style)" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root.set("tags", ["ruby", "rails"], style: :flow)

    assert_equal <<~YAML, document.to_s
      name: Alice
      tags: [ruby, rails]
    YAML
  end

  test "[]= sets empty hash" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["settings"] = {}

    assert_equal <<~YAML, document.to_s
      name: Alice
      settings: {}
    YAML
  end

  test "[]= sets hash with values (block by default)" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["settings"] = { theme: "dark", lang: "en" }

    assert_equal <<~YAML, document.to_s
      name: Alice
      settings:
        theme: dark
        lang: en
    YAML
  end

  test "[]= sets hash with values (flow style)" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root.set("settings", { theme: "dark", lang: "en" }, style: :flow)

    assert_equal <<~YAML, document.to_s
      name: Alice
      settings: {theme: dark, lang: en}
    YAML
  end

  test "insert with empty array" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      age: 30
    YAML
    document.root.insert("speakers", [], after: "name")

    assert_equal <<~YAML, document.to_s
      name: Alice
      speakers: []
      age: 30
    YAML
  end

  test "insert with block style array" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      age: 30
    YAML
    document.root.insert("tags", ["ruby", "rails"], after: "name")

    assert_equal <<~YAML, document.to_s
      name: Alice
      tags:
        - ruby
        - rails
      age: 30
    YAML
  end

  test "[]= replaces existing value with array (block by default)" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      tags: old
    YAML
    document.root["tags"] = ["new"]

    assert_equal <<~YAML, document.to_s
      name: Alice
      tags:
        - new
    YAML
  end

  test "collection_style reads flow" do
    document = Yerba::Document.parse(<<~YAML)
      tags: [ruby, rails]
    YAML

    assert_equal :flow, document["tags"].collection_style
  end

  test "collection_style reads block" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rails
    YAML

    assert_equal :block, document["tags"].collection_style
  end

  test "collection_style= converts flow to block" do
    document = Yerba::Document.parse(<<~YAML)
      tags: [ruby, rails]
    YAML
    document["tags"].collection_style = :block

    assert_equal <<~YAML, document.to_s
      tags:
        - ruby
        - rails
    YAML
  end

  test "collection_style= converts block to flow" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rails
    YAML
    document["tags"].collection_style = :flow

    assert_equal <<~YAML, document.to_s
      tags: [ruby, rails]
    YAML
  end

  test "collection_style reads flow map" do
    document = Yerba::Document.parse(<<~YAML)
      database: {host: localhost, port: 5432}
    YAML

    assert_equal :flow, document["database"].collection_style
  end

  test "collection_style reads block map" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal :block, document["database"].collection_style
  end

  test "collection_style= converts flow map to block" do
    document = Yerba::Document.parse(<<~YAML)
      database: {host: localhost, port: 5432}
    YAML
    document["database"].collection_style = :block

    assert_equal <<~YAML, document.to_s
      database:
        host: localhost
        port: 5432
    YAML
  end

  test "collection_style= converts block map to flow" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML
    document["database"].collection_style = :flow

    assert_equal <<~YAML, document.to_s
      database: {host: localhost, port: 5432}
    YAML
  end

  test "[]= sets nested hash with block style" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["config"] = { database: { host: "localhost", port: 5432 } }

    assert_equal <<~YAML, document.to_s
      name: Alice
      config:
        database:
          host: localhost
          port: 5432
    YAML
  end

  test "[]= sets nested array with block style" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root["speakers"] = [{ name: "Bob" }, { name: "Carol" }]

    assert_equal <<~YAML, document.to_s
      name: Alice
      speakers:
        - name: Bob
        - name: Carol
    YAML
  end

  test "set with style: :flow for nested structure" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.root.set("config", { host: "localhost", port: 5432 }, style: :flow)

    assert_equal <<~YAML, document.to_s
      name: Alice
      config: {host: localhost, port: 5432}
    YAML
  end

  test "collection_style on nested sequence" do
    document = Yerba::Document.parse(<<~YAML)
      app:
        tags:
          - ruby
          - rails
    YAML

    assert_equal :block, document["app"]["tags"].collection_style
  end

  test "collection_style= on nested sequence" do
    document = Yerba::Document.parse(<<~YAML)
      app:
        tags: [ruby, rails]
    YAML
    document["app"]["tags"].collection_style = :block

    assert_equal <<~YAML, document.to_s
      app:
        tags:
          - ruby
          - rails
    YAML
  end
end
