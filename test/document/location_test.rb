# frozen_string_literal: true

require "test_helper"

module Document
  class LocationTest < Minitest::Spec
    test "document.location returns whole document location" do
      document = Yerba::Document.parse(<<~YAML)
        name: Alice
        port: 5432
      YAML

      loc = document.location

      assert_instance_of Yerba::Location, loc
      assert_equal 1, loc.start_line
      assert_equal 0, loc.start_column
    end

    test "document.location with selector returns location of value" do
      document = Yerba::Document.parse(<<~YAML)
        name: Alice
        port: 5432
      YAML

      loc = document.location("port")

      assert_equal 2, loc.start_line
      assert_equal 6, loc.start_column
    end

    test "document.location with nested selector" do
      document = Yerba::Document.parse(<<~YAML)
        database:
          host: localhost
      YAML

      loc = document.location("database.host")

      assert_equal 2, loc.start_line
    end

    test "document.location with bracket selector" do
      document = Yerba::Document.parse(<<~YAML)
        - id: talk-1
          title: First
        - id: talk-2
          title: Second
      YAML

      loc = document.location("[1].title")

      assert_equal 4, loc.start_line
      assert_equal 4, loc.end_line
      assert_equal 9, loc.start_column
      assert_equal 15, loc.end_column
    end

    test "document.location returns nil for missing selector" do
      document = Yerba::Document.parse(<<~YAML)
        name: Alice
      YAML

      assert_nil document.location("nonexistent")
    end

    test "document.location spans multiple lines for block scalars" do
      document = Yerba::Document.parse(<<~YAML)
        - id: talk-1
          description: |-
            First line.
            Second line.
            Third line.
        - id: talk-2
      YAML

      loc = document.location("[0].description")

      assert_equal 2, loc.start_line
      assert_equal 5, loc.end_line
      assert_equal 15, loc.start_column
      assert_equal 15, loc.end_column
    end

    test "document.location for single line value has same start and end line" do
      document = Yerba::Document.parse(<<~YAML)
        name: Alice
      YAML

      loc = document.location("name")

      assert_equal loc.start_line, loc.end_line
    end

    test "document.location for map spans all its keys" do
      document = Yerba::Document.parse(<<~YAML)
        - id: talk-1
          title: First
          description: |-
            Some text.
            More text.
        - id: talk-2
      YAML

      loc = document.location("[0]")

      assert_equal 1, loc.start_line
      assert_equal 5, loc.end_line
    end

    test "document.location has byte offsets" do
      document = Yerba::Document.parse(<<~YAML)
        name: Alice
      YAML

      loc = document.location("name")

      assert_equal 6, loc.start_offset
      assert_equal 11, loc.end_offset
    end

    test "document.locations returns array for wildcard selectors" do
      document = Yerba::Document.parse(<<~YAML)
        - id: a
          title: First
        - id: b
          title: Second
      YAML

      locs = document.locations("[].title")

      assert_equal 2, locs.length
      assert_equal 2, locs[0].start_line
      assert_equal 4, locs[1].start_line
    end

    test "document.locations returns all items" do
      document = Yerba::Document.parse(<<~YAML)
        - id: a
        - id: b
        - id: c
      YAML

      locs = document.locations("[]")

      assert_equal 3, locs.length
      assert_equal 1, locs[0].start_line
      assert_equal 2, locs[1].start_line
      assert_equal 3, locs[2].start_line
    end

    test "document.locations with nested wildcards" do
      document = Yerba::Document.parse(<<~YAML)
        - speakers:
            - Alice
            - Bob
        - speakers:
            - Charlie
      YAML

      locs = document.locations("[].speakers[]")

      assert_equal 3, locs.length
      assert_equal 2, locs[0].start_line
      assert_equal 3, locs[1].start_line
      assert_equal 5, locs[2].start_line
    end

    test "document.locations returns empty array for no matches" do
      document = Yerba::Document.parse(<<~YAML)
        name: Alice
      YAML

      locs = document.locations("[].nonexistent")

      assert_equal [], locs
    end
  end
end
