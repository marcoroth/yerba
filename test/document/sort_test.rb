# frozen_string_literal: true

require "test_helper"

module Document
  class SortTest < Minitest::Spec
    test "sort orders sequence items" do
      document = Yerba::Document.parse(<<~YAML)
        tags:
          - rust
          - go
          - ruby
      YAML

      document.sort("tags")

      assert_equal <<~YAML, document.to_s
        tags:
          - go
          - ruby
          - rust
      YAML
    end

    test "document.sort without path sorts root sequence" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Charlie
        - name: Alice
        - name: Bob
      YAML

      document.sort(by: "name")

      assert_equal "Alice", document.value_at("[0].name")
      assert_equal "Bob", document.value_at("[1].name")
      assert_equal "Charlie", document.value_at("[2].name")
    end

    test "document.sort with order: :desc sorts descending" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Alice
        - name: Charlie
        - name: Bob
      YAML

      document.sort(by: "name", order: :desc)

      assert_equal "Charlie", document.value_at("[0].name")
      assert_equal "Bob", document.value_at("[1].name")
      assert_equal "Alice", document.value_at("[2].name")
    end

    test "document.sort with order: 'desc' accepts string" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Alice
        - name: Charlie
        - name: Bob
      YAML

      document.sort(by: "name", order: "desc")

      assert_equal "Charlie", document.value_at("[0].name")
      assert_equal "Bob", document.value_at("[1].name")
      assert_equal "Alice", document.value_at("[2].name")
    end

    test "document.sort with order: array reorders explicitly" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Alice
        - name: Charlie
        - name: Bob
      YAML

      document.sort(by: "name", order: ["Charlie", "Bob", "Alice"])

      assert_equal "Charlie", document.value_at("[0].name")
      assert_equal "Bob", document.value_at("[1].name")
      assert_equal "Alice", document.value_at("[2].name")
    end

    test "document.sort with order: array raises on missing values" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Alice
        - name: Charlie
        - name: Bob
      YAML

      error = assert_raises(Yerba::Error) do
        document.sort(by: "name", order: ["Charlie", "Alice"])
      end

      assert_match(/must specify all 3 items/, error.message)
      assert_match(/Bob/, error.message)
    end

    test "document.sort with order: array raises on unknown value" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Alice
        - name: Charlie
        - name: Bob
      YAML

      error = assert_raises(Yerba::Error) do
        document.sort(by: "name", order: ["Charlie", "Dave", "Alice"])
      end

      assert_match(/no item found with name == "Dave"/, error.message)
    end

    test "document.sort with order: array on maps without by: raises" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Alice
        - name: Bob
      YAML

      assert_raises(Yerba::Error) do
        document.sort(order: ["Bob", "Alice"])
      end
    end

    test "document.sort with by: as symbol" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Charlie
        - name: Alice
        - name: Bob
      YAML

      document.sort(by: :name, order: :desc)

      assert_equal "Charlie", document.value_at("[0].name")
      assert_equal "Bob", document.value_at("[1].name")
      assert_equal "Alice", document.value_at("[2].name")
    end

    test "document.sort with by: as symbol and array order" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Alice
        - name: Charlie
        - name: Bob
      YAML

      document.sort(by: :name, order: ["Bob", "Alice", "Charlie"])

      assert_equal "Bob", document.value_at("[0].name")
      assert_equal "Alice", document.value_at("[1].name")
      assert_equal "Charlie", document.value_at("[2].name")
    end

    test "document.sort with by: using dot prefix" do
      document = Yerba::Document.parse(<<~YAML)
        - name: Charlie
        - name: Alice
        - name: Bob
      YAML

      document.sort(by: ".name", order: :desc)

      assert_equal "Charlie", document.value_at("[0].name")
      assert_equal "Bob", document.value_at("[1].name")
      assert_equal "Alice", document.value_at("[2].name")
    end

    test "document.sort scalar sequence ascending" do
      document = Yerba::Document.parse(<<~YAML)
        - rust
        - go
        - ruby
      YAML

      document.sort

      assert_equal "go", document.value_at("[0]")
      assert_equal "ruby", document.value_at("[1]")
      assert_equal "rust", document.value_at("[2]")
    end

    test "document.sort scalar sequence descending" do
      document = Yerba::Document.parse(<<~YAML)
        - go
        - ruby
        - rust
      YAML

      document.sort(order: :desc)

      assert_equal "rust", document.value_at("[0]")
      assert_equal "ruby", document.value_at("[1]")
      assert_equal "go", document.value_at("[2]")
    end

    test "document.sort nested scalar sequence descending" do
      document = Yerba::Document.parse(<<~YAML)
        tags:
          - go
          - ruby
          - rust
      YAML

      document.sort("tags", order: :desc)

      assert_equal "rust", document.value_at("tags[0]")
      assert_equal "ruby", document.value_at("tags[1]")
      assert_equal "go", document.value_at("tags[2]")
    end

    test "document.sort nested scalar sequence with explicit order" do
      document = Yerba::Document.parse(<<~YAML)
        tags:
          - ruby
          - go
          - rust
      YAML

      document.sort("tags", order: ["rust", "ruby", "go"])

      assert_equal "rust", document.value_at("tags[0]")
      assert_equal "ruby", document.value_at("tags[1]")
      assert_equal "go", document.value_at("tags[2]")
    end

    test "document.sort root scalar sequence with explicit order" do
      document = Yerba::Document.parse(<<~YAML)
        - ruby
        - go
        - rust
      YAML

      document.sort(order: ["rust", "ruby", "go"])

      assert_equal "rust", document.value_at("[0]")
      assert_equal "ruby", document.value_at("[1]")
      assert_equal "go", document.value_at("[2]")
    end

    test "document.sort with path sorts nested sequence" do
      document = Yerba::Document.parse(<<~YAML)
        items:
          - name: Charlie
          - name: Alice
      YAML

      document.sort("items", by: "name")

      assert_equal "Alice", document.value_at("items[0].name")
      assert_equal "Charlie", document.value_at("items[1].name")
    end
  end
end
