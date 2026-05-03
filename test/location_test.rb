# frozen_string_literal: true

require "test_helper"

class LocationTest < Minitest::Spec
  test "scalar has value location" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    scalar = document["name"]

    assert_instance_of Yerba::Location, scalar.location
    assert_equal 1, scalar.location.start_line
    assert_equal 6, scalar.location.start_column
  end

  test "scalar has key" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    scalar = document["name"]

    assert_instance_of Yerba::Scalar, scalar.key
    assert_equal "name", scalar.key.value
  end

  test "scalar key has location" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    scalar = document["name"]

    assert_instance_of Yerba::Location, scalar.key.location
    assert_equal 1, scalar.key.location.start_line
    assert_equal 0, scalar.key.location.start_column
    assert_equal 4, scalar.key.location.end_column
  end

  test "nested scalar has correct locations" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    host = document["database"]["host"]

    assert_equal 2, host.location.start_line
    assert_equal 8, host.location.start_column
    assert_equal 2, host.key.location.start_line
    assert_equal 2, host.key.location.start_column
    assert_equal "host", host.key.value
  end

  test "selector returns the path string" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_equal "database.host", document["database"]["host"].selector
    assert_equal "database", document["database"].selector
  end

  test "map has location" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    map = document["database"]

    assert_instance_of Yerba::Location, map.location
    assert_equal 2, map.location.start_line
  end

  test "map has key" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    map = document["database"]

    assert_instance_of Yerba::Scalar, map.key
    assert_equal "database", map.key.value
  end

  test "map key has location" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    map = document["database"]

    assert_equal 1, map.key.location.start_line
    assert_equal 0, map.key.location.start_column
    assert_equal 8, map.key.location.end_column
  end

  test "map selector returns path string" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    assert_equal "database", document["database"].selector
  end

  test "sequence has location" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
        - rust
    YAML

    seq = document["tags"]

    assert_instance_of Yerba::Location, seq.location
    assert_equal 2, seq.location.start_line
  end

  test "sequence has key" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML

    seq = document["tags"]

    assert_instance_of Yerba::Scalar, seq.key
    assert_equal "tags", seq.key.value
  end

  test "sequence key has location" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML

    seq = document["tags"]

    assert_equal 1, seq.key.location.start_line
    assert_equal 0, seq.key.location.start_column
    assert_equal 4, seq.key.location.end_column
  end

  test "sequence selector returns path string" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML

    assert_equal "tags", document["tags"].selector
  end

  test "indexed sequence item has location" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: Ruby
        - name: Rust
    YAML

    item = document["items"][1]

    assert_instance_of Yerba::Location, item.location
    assert_equal 3, item.location.start_line
  end

  test "inserted scalar has location" do
    document = Yerba::Document.parse(<<~YAML)
      tags:
        - ruby
    YAML

    document["tags"] << "rust"
    scalar = document["tags"][1]

    assert_instance_of Yerba::Location, scalar.location
    assert_equal 3, scalar.location.start_line
  end

  test "inserted map entry has location" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - name: "Ruby"
    YAML

    document["items"] << { name: "Rust" }
    item = document["items"][1]

    assert_instance_of Yerba::Location, item.location
    assert_equal 3, item.location.start_line
  end

  test "inserted key has location" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
    YAML

    document["database"].insert("port", "5432")
    port = document["database"]["port"]

    assert_instance_of Yerba::Location, port.location
    assert_equal 3, port.location.start_line
    assert_equal "port", port.key.value
    assert_equal 3, port.key.location.start_line
  end

  test "location updates after set" do
    document = Yerba::Document.parse(<<~YAML)
      name: short
    YAML

    document.set("name", "a much longer value")
    scalar = document["name"]

    assert_equal 1, scalar.location.start_line
    assert_equal 25, scalar.location.end_column
  end
end
