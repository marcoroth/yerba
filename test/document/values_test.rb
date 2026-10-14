# frozen_string_literal: true

require "test_helper"
require "tempfile"

module Document
  class ValuesTest < Minitest::Spec
    def speakers
      Yerba::Document.parse(<<~YAML)
        - name: Alice
          github: alice
        - name: Bob
      YAML
    end

    test "values returns the whole document as Ruby objects" do
      assert_equal [{ "name" => "Alice", "github" => "alice" }, { "name" => "Bob" }], speakers.values
    end

    test "values returns the same object until the document changes" do
      document = speakers

      assert_same document.values, document.values
    end

    test "values reflects a set on the document" do
      document = speakers
      document.values

      document.set("[1].name", "Robert")

      assert_equal "Robert", document.values[1]["name"]
    end

    test "values reflects an edit made through a node" do
      document = speakers
      document.values

      document.root[0]["name"] = "Alicia"

      assert_equal "Alicia", document.values[0]["name"]
    end

    test "values reflects appending, deleting and sorting" do
      document = speakers
      document.values

      document << { "name" => "Aaron" }
      assert_equal(["Alice", "Bob", "Aaron"], document.values.map { |entry| entry["name"] })

      document.root.delete_at(1)
      assert_equal(["Alice", "Aaron"], document.values.map { |entry| entry["name"] })

      document.sort(by: :name)
      assert_equal(["Aaron", "Alice"], document.values.map { |entry| entry["name"] })
    end

    test "values reflects replacing the root" do
      document = speakers
      document.values

      document.root = { "name" => "Carol" }

      assert_equal({ "name" => "Carol" }, document.values)
    end

    test "values is cached per document" do
      first = speakers
      second = speakers

      first.values
      second.set("[0].name", "Changed")

      assert_equal "Alice", first.values[0]["name"]
      assert_equal "Changed", second.values[0]["name"]
    end

    test "values of an empty document is nil" do
      assert_nil Yerba::Document.parse("").values
    end

    test "values reads a parsed file" do
      file = Tempfile.new(["speakers", ".yml"])
      file.write("- name: Alice\n")
      file.close

      assert_equal [{ "name" => "Alice" }], Yerba.parse_file(file.path).values
    ensure
      file&.unlink
    end

    test "values passes its JSON options as keywords" do
      original = JSON.method(:parse)
      JSON.define_singleton_method(:parse) { |source, **options| original.call(source, **options) }

      assert_equal({ "a" => Float::INFINITY }, Yerba.parse("a: .inf\n").values)
    ensure
      JSON.define_singleton_method(:parse, original)
    end
  end
end
