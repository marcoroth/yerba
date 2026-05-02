# frozen_string_literal: true

require "test_helper"

class ScalarTest < Minitest::Spec
  test "[] on scalar path returns Yerba::Scalar" do
    document = Yerba::Document.parse("database:\n  host: localhost")

    assert_instance_of Yerba::Scalar, document["database"]["host"]
  end

  test "scalar.value returns typed value" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert_equal "localhost", document["database"]["host"].value
    assert_equal 5432, document["database"]["port"].value
  end

  test "scalar.quote_style returns style" do
    document = Yerba::Document.parse("plain: hello\nquoted: \"world\"")

    assert_equal :plain, document["plain"].quote_style
    assert_equal :double, document["quoted"].quote_style
  end

  test "scalar.quote_style= changes quote style" do
    document = Yerba::Document.parse("name: Alice")
    document["name"].quote_style = :double

    assert_includes document.to_s, '"Alice"'
  end

  test "scalar.to_s returns string value" do
    document = Yerba::Document.parse("name: Alice")

    assert_equal "Alice", document["name"].to_s
  end

  test "scalar == compares value" do
    document = Yerba::Document.parse("name: Alice")

    assert_equal "Alice", document["name"]
  end

  test "scalar inspect shows path and value" do
    document = Yerba::Document.parse("name: Alice")

    assert_equal '#<Yerba::Scalar path="name" value="Alice">', document["name"].inspect
  end
end
