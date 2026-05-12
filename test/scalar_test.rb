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

    assert_equal "Bob", scalar2.document.get("name")
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
