# frozen_string_literal: true

require "test_helper"

class ScalarTest < Minitest::Spec
  test "[] on scalar path returns Yerba::Scalar" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_instance_of Yerba::Scalar, document["database"]["host"]
  end

  test "scalar.value returns typed value" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert_equal "localhost", document["database"]["host"].value
    assert_equal 5432, document["database"]["port"].value
  end

  test "scalar.quote_style returns style" do
    document = Yerba::Document.parse(<<~YAML)
      plain: hello
      quoted: "world"
    YAML

    assert_equal :plain, document["plain"].quote_style
    assert_equal :double, document["quoted"].quote_style
  end

  test "scalar.quote_style= changes quote style" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document["name"].quote_style = :double

    assert_equal <<~YAML, document.to_s
      name: "Alice"
    YAML
  end

  test "scalar.to_s returns string value" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal "Alice", document["name"].to_s
  end

  test "scalar == compares value" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal "Alice", document["name"]
  end

  test "scalar.quote_style= plain to double" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document["name"].quote_style = :double

    assert_equal :double, document["name"].quote_style
    assert_equal <<~YAML, document.to_s
      name: "Alice"
    YAML
  end

  test "scalar.quote_style= double to single" do
    document = Yerba::Document.parse(<<~YAML)
      name: "Alice"
    YAML
    document["name"].quote_style = :single

    assert_equal :single, document["name"].quote_style
    assert_equal <<~YAML, document.to_s
      name: 'Alice'
    YAML
  end

  test "scalar.quote_style= double to plain" do
    document = Yerba::Document.parse(<<~YAML)
      name: "Alice"
    YAML
    document["name"].quote_style = :plain

    assert_equal :plain, document["name"].quote_style
    assert_equal <<~YAML, document.to_s
      name: Alice
    YAML
  end

  test "scalar.quote_style= single to double" do
    document = Yerba::Document.parse(<<~YAML)
      name: 'Alice'
    YAML
    document["name"].quote_style = :double

    assert_equal :double, document["name"].quote_style
    assert_equal <<~YAML, document.to_s
      name: "Alice"
    YAML
  end

  test "scalar.quote_style= preserves other values" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      age: 30
    YAML
    document["name"].quote_style = :double

    assert_equal <<~YAML, document.to_s
      name: "Alice"
      age: 30
    YAML
  end

  test "scalar inspect shows path and value" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal '#<Yerba::Scalar selector="name" value="Alice">', document["name"].inspect
  end

  test "Scalar.new standalone with value" do
    scalar = Yerba::Scalar.new("hello")

    assert_equal "hello", scalar.value
    assert_nil scalar.selector
    assert_nil scalar.file_path
    assert_nil scalar.line
    refute scalar.connected?
  end

  test "Scalar.new standalone with quote_style" do
    scalar = Yerba::Scalar.new("hello", quote_style: :double)

    assert_equal "hello", scalar.value
    assert_equal :double, scalar.quote_style
  end

  test "Scalar.from creates scalar with metadata" do
    scalar = Yerba::Scalar.from(
      file_path: "/tmp/test.yml",
      selector: "[0].name",
      line: 5,
      value: "Alice"
    )

    assert_equal "Alice", scalar.value
    assert_equal "/tmp/test.yml", scalar.file_path
    assert_equal "[0].name", scalar.selector
    assert_equal 5, scalar.line
    assert scalar.connected?
  end

  test "Scalar.from without value lazily loads from document" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    scalar = Yerba::Scalar.from_document(document, "name")

    assert_equal "Alice", scalar.value
    assert_equal "name", scalar.selector
    assert scalar.connected?
  end

  test "Scalar.from_document with value and location" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    scalar = document["name"]

    assert_equal "Alice", scalar.value
    assert_equal "name", scalar.selector
    assert scalar.location
    assert scalar.connected?
  end

  test "Scalar.from lazily loads document on value read" do
    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\n")
    file.close

    scalar = Yerba::Scalar.from(file_path: file.path, selector: "name")

    assert_nil scalar.instance_variable_get(:@document)
    assert_equal "Alice", scalar.value
    refute_nil scalar.document
  ensure
    file&.unlink
  end

  test "Scalar.from lazily loads document on mutation" do
    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\n")
    file.close

    scalar = Yerba::Scalar.from(file_path: file.path, selector: "name")

    assert_nil scalar.instance_variable_get(:@document)

    scalar.value = "Bob"

    refute_nil scalar.document
    assert_equal "Bob", scalar.value
    assert_equal <<~YAML, scalar.document.to_s
      name: Bob
    YAML
  ensure
    file&.unlink
  end

  test "Scalar.from shares document across scalars from same file" do
    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\nport: 5432\n")
    file.close

    scalar1 = Yerba::Scalar.from(file_path: file.path, selector: "name")
    scalar2 = Yerba::Scalar.from(file_path: file.path, selector: "port")

    scalar1.value
    scalar2.value

    assert_same scalar1.document, scalar2.document
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end

  test "Scalar.from mutation is visible to other scalars from same file" do
    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\nport: 5432\n")
    file.close

    scalar1 = Yerba::Scalar.from(file_path: file.path, selector: "name")
    scalar2 = Yerba::Scalar.from(file_path: file.path, selector: "name")

    scalar1.value = "Bob"

    assert_equal "Bob", scalar2.document.value_at("name")
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end

  test "Scalar.from quote_style mutation lazily loads document" do
    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\n")
    file.close

    scalar = Yerba::Scalar.from(file_path: file.path, selector: "name")
    scalar.quote_style = :double

    assert_equal :double, scalar.quote_style
    assert_includes scalar.document.to_s, '"Alice"'
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end

  test "scalar.value reflects later document edits" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]

    assert_equal "localhost", scalar.value

    document.set("host", "0.0.0.0")

    assert_equal "0.0.0.0", scalar.value
  end

  test "scalar.value reflects later document edits for falsy values" do
    document = Yerba::Document.parse(<<~YAML)
      enabled: true
      name: Alice
    YAML

    enabled = document["enabled"]
    name = document["name"]

    assert_equal true, enabled.value
    assert_equal "Alice", name.value

    document.set("enabled", false)
    document.set("name", nil)

    assert_equal false, enabled.value
    assert_nil name.value
  end

  test "scalar.value reflects edits made through another scalar" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]
    other = document["host"]

    other.value = "0.0.0.0"

    assert_equal "0.0.0.0", scalar.value
  end

  test "scalar.value is nil after the node is deleted" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
      port: 5432
    YAML

    scalar = document["host"]

    assert_equal "localhost", scalar.value

    document.delete("host")

    assert_nil scalar.value
  end

  test "scalar.value reflects file edits made through another scalar" do
    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\n")
    file.close

    scalar = Yerba::Scalar.from(file_path: file.path, selector: "name", value: "Alice")
    other = Yerba::Scalar.from(file_path: file.path, selector: "name")

    other.value = "Bob"

    assert_equal "Bob", scalar.value
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end

  test "scalar.value falls back to the captured value when the file is gone" do
    scalar = Yerba::Scalar.from(file_path: "/tmp/does-not-exist.yml", selector: "name", value: "Alice")

    assert_equal "Alice", scalar.value
  end

  test "scalar.value reflects edits made through document[]=" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]

    assert_equal "localhost", scalar.value

    document["host"] = "0.0.0.0"

    assert_equal "0.0.0.0", scalar.value
  end

  test "scalar.value reflects edits made through the enclosing map" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    scalar = document["database"]["host"]

    assert_equal "localhost", scalar.value

    document["database"]["host"] = "0.0.0.0"

    assert_equal "0.0.0.0", scalar.value
  end

  test "scalar.value reflects edits made to a sibling key" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
      port: 5432
    YAML

    host = document["host"]

    assert_equal "localhost", host.value

    document.set("port", 5433)

    assert_equal "localhost", host.value
    assert_equal 5433, document["port"].value
  end

  test "scalar.value reflects a key that is deleted and added back" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
      port: 5432
    YAML

    scalar = document["host"]

    assert_equal "localhost", scalar.value

    document.delete("host")

    assert_nil scalar.value

    document["host"] = "0.0.0.0"

    assert_equal "0.0.0.0", scalar.value
  end

  test "scalar.value reflects appends to the enclosing sequence" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: first
    YAML

    scalar = document["items"][0]["name"]

    assert_equal "first", scalar.value

    document["items"] << { "name" => "second" }

    assert_equal "first", scalar.value
    assert_equal "second", document["items"][1]["name"].value
  end

  test "scalar.value follows its index when sequence items are removed" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: first
        - name: second
    YAML

    scalar = document["items"][1]["name"]

    assert_equal "second", scalar.value

    document["items"].delete_at(0)

    assert_nil scalar.value
    assert_equal "second", document["items"][0]["name"].value
  end

  test "scalar.value reflects renamed keys" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]

    assert_equal "localhost", scalar.value

    document.rename("host", "hostname")

    assert_nil scalar.value
    assert_equal "localhost", document["hostname"].value
  end

  test "scalar.value reflects sorted keys" do
    document = Yerba::Document.parse(<<~YAML)
      port: 5432
      host: localhost
    YAML

    scalar = document["host"]

    assert_equal "localhost", scalar.value

    document.sort_keys("", ["host", "port"])

    assert_equal "localhost", scalar.value
  end

  test "scalar.value is unaffected by a quote style change" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    scalar = document["name"]

    assert_equal "Alice", scalar.value

    scalar.quote_style = :double

    assert_equal "Alice", scalar.value
    assert_equal :double, scalar.quote_style
  end

  test "scalar.value is nil after the scalar deletes itself" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
      port: 5432
    YAML

    scalar = document["host"]

    assert_equal "localhost", scalar.value

    scalar.delete

    assert_nil scalar.value
  end

  test "scalar conversions reflect later document edits" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      port: 5432
    YAML

    name = document["name"]
    port = document["port"]

    assert_equal "Alice", name.to_s
    assert_equal "Alice", name.to_yaml
    assert_equal 5432, port.to_i
    assert_equal 5432.0, port.to_f
    assert_equal "Alice", name

    document.set("name", "Bob")
    document.set("port", 5433)

    assert_equal "Bob", name.to_s
    assert_equal "Bob", name.to_yaml
    assert_equal 5433, port.to_i
    assert_equal 5433.0, port.to_f
    assert_equal "Bob", name
    refute_equal "Alice", name
  end

  test "scalar.inspect reflects later document edits" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]

    assert_includes scalar.inspect, "localhost"

    document.set("host", "0.0.0.0")

    assert_includes scalar.inspect, "0.0.0.0"
    refute_includes scalar.inspect, "localhost"
  end

  test "key scalars keep their value across document edits" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]

    assert_equal "host", scalar.key.value

    document.set("host", "0.0.0.0")

    assert_equal "host", scalar.key.value
  end

  test "standalone scalars keep their value" do
    scalar = Yerba::Scalar.new("hello")

    assert_equal "hello", scalar.value
    assert_equal "hello", scalar.value

    scalar.value = "goodbye"

    assert_equal "goodbye", scalar.value
  end

  test "scalars from the same document all see an edit" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalars = Array.new(3) { document["host"] }

    scalars.each { |scalar| assert_equal "localhost", scalar.value }

    document.set("host", "0.0.0.0")

    scalars.each { |scalar| assert_equal "0.0.0.0", scalar.value }
  end

  test "scalars from different documents cache independently" do
    first = Yerba::Document.parse(<<~YAML)
      host: first
    YAML

    second = Yerba::Document.parse(<<~YAML)
      host: second
    YAML

    first_scalar = first["host"]
    second_scalar = second["host"]

    first.set("host", "changed")

    assert_equal "changed", first_scalar.value
    assert_equal "second", second_scalar.value
  end

  test "scalar.value re-reads once per edit" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]
    reads = 0

    document.define_singleton_method(:value_at) do |selector|
      reads += 1
      super(selector)
    end

    5.times do |index|
      document.set("host", "host#{index}")

      3.times { assert_equal "host#{index}", scalar.value }
    end

    assert_equal 5, reads
  end

  test "scalar.value reuses the cached value while the document is unchanged" do
    document = Yerba::Document.parse(<<~YAML)
      host: localhost
    YAML

    scalar = document["host"]
    reads = 0

    document.define_singleton_method(:value_at) do |selector|
      reads += 1
      super(selector)
    end

    3.times { assert_equal "localhost", scalar.value }

    assert_equal 0, reads

    document.set("host", "0.0.0.0")

    3.times { assert_equal "0.0.0.0", scalar.value }

    assert_equal 1, reads
  end

  test "scalar.value reflects content replaced wholesale" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    scalar = document["name"]

    assert_equal "Alice", scalar.value

    document.replace_content!("name: Bob\n")

    assert_equal "Bob", scalar.value
  end

  test "Scalar.from delete lazily loads document" do
    file = Tempfile.new(["test", ".yml"])
    file.write("name: Alice\nport: 5432\n")
    file.close

    scalar = Yerba::Scalar.from(file_path: file.path, selector: "name")
    scalar.delete

    refute_includes scalar.document.to_s, "name"
    assert_includes scalar.document.to_s, "port"
  ensure
    file&.unlink
    Yerba::Document.clear_cache!
  end
end
