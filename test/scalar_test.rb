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

    assert_includes document.to_s, '"Alice"'
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

  test "scalar inspect shows path and value" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_equal '#<Yerba::Scalar path="name" value="Alice">', document["name"].inspect
  end
end
