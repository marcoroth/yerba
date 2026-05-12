# frozen_string_literal: true

require "test_helper"
require "tmpdir"
require "fileutils"

class CollectionTest < Minitest::Spec
  def setup
    @dir = Dir.mktmpdir("yerba_test")
    File.write(File.join(@dir, "a.yml"), "name: Alpha\nport: 1000\n")
    File.write(File.join(@dir, "b.yml"), "name: Beta\nport: 2000\n")
    File.write(File.join(@dir, "c.yml"), "items:\n  - name: One\n  - name: Two\n")
  end

  def teardown
    FileUtils.rm_rf(@dir)
  end

  test "each iterates over matching files" do
    collection = Yerba.files(File.join(@dir, "*.yml"))
    paths = collection.map(&:path)

    assert_equal 3, paths.length
  end

  test "get returns Scalar objects with value, file_path, line, and selector" do
    collection = Yerba.files(File.join(@dir, "[ab].yml"))
    result = collection.get("name")

    assert_equal 2, result.length
    assert_instance_of Yerba::Scalar, result.first

    values = result.map(&:value).sort
    assert_equal ["Alpha", "Beta"], values

    result.each do |scalar|
      assert scalar.file_path
      assert scalar.line
      assert_equal "name", scalar.selector
    end
  end

  test "get with wildcard returns Scalars with resolved selectors" do
    collection = Yerba.files(File.join(@dir, "c.yml"))
    result = collection.get("items[].name")

    values = result.map(&:value)
    assert_equal ["One", "Two"], values

    selectors = result.map(&:selector)
    assert_equal ["items[0].name", "items[1].name"], selectors
  end

  test "get with nested wildcards returns correct selectors" do
    File.write(File.join(@dir, "nested.yml"), <<~YAML)
      - id: video-1
        speakers:
          - Alice
          - Bob
      - id: video-2
        speakers:
          - Charlie
    YAML

    result = Yerba::Collection.get(File.join(@dir, "nested.yml"), "[].speakers[]")

    assert_equal 3, result.length

    assert_equal "Alice", result[0].value
    assert_equal "[0].speakers[0]", result[0].selector

    assert_equal "Bob", result[1].value
    assert_equal "[0].speakers[1]", result[1].selector

    assert_equal "Charlie", result[2].value
    assert_equal "[1].speakers[0]", result[2].selector
  end

  test "get includes file_path for each result" do
    result = Yerba::Collection.get(File.join(@dir, "[ab].yml"), "name")

    file_paths = result.map(&:file_path).sort
    assert_equal [File.join(@dir, "a.yml"), File.join(@dir, "b.yml")], file_paths
  end

  test "get includes line numbers" do
    File.write(File.join(@dir, "lines.yml"), <<~YAML)
      host: localhost
      port: 5432
    YAML

    result = Yerba::Collection.get(File.join(@dir, "lines.yml"), "port")

    assert_equal 1, result.length
    assert_equal 5432, result[0].value
    assert_equal 2, result[0].line
  end

  test "get returns Map objects for map selectors" do
    collection = Yerba.files(File.join(@dir, "c.yml"))
    result = collection.get("items[]")

    assert_equal 2, result.length
    assert_instance_of Yerba::Map, result.first
    assert_equal "One", result[0]["name"].value
    assert_equal "Two", result[1]["name"].value
  end

  test "get returns Map objects with file_path, line, and selector" do
    collection = Yerba.files(File.join(@dir, "c.yml"))
    result = collection.get("items[]")

    assert_equal File.join(@dir, "c.yml"), result[0].file_path
    assert_equal "items[0]", result[0].selector
    assert result[0].line
  end

  test "get returns Sequence objects for sequence selectors" do
    File.write(File.join(@dir, "talks.yml"), <<~YAML)
      - id: talk-1
        speakers:
          - Alice
          - Bob
      - id: talk-2
        speakers:
          - Charlie
    YAML

    result = Yerba::Collection.get(File.join(@dir, "talks.yml"), "[].speakers")

    assert_equal 2, result.length
    assert_instance_of Yerba::Sequence, result.first
    assert_equal "[0].speakers", result[0].selector
    assert_equal "[1].speakers", result[1].selector
  end

  test "get returns consistent types across glob" do
    File.write(File.join(@dir, "d.yml"), <<~YAML)
      - id: talk-1
        title: First
      - id: talk-2
        title: Second
    YAML
    File.write(File.join(@dir, "e.yml"), <<~YAML)
      - id: talk-3
        title: Third
    YAML

    maps = Yerba::Collection.get(File.join(@dir, "[de].yml"), "[]")
    assert(maps.all?(Yerba::Map))
    assert_equal 3, maps.length

    scalars = Yerba::Collection.get(File.join(@dir, "[de].yml"), "[].title")
    assert(scalars.all?(Yerba::Scalar))
    assert_equal ["First", "Second", "Third"], scalars.map(&:value)
  end

  test "get maps have correct file_path across glob" do
    File.write(File.join(@dir, "d.yml"), "- id: d1\n")
    File.write(File.join(@dir, "e.yml"), "- id: e1\n")

    result = Yerba::Collection.get(File.join(@dir, "[de].yml"), "[]")

    file_paths = result.map(&:file_path).sort
    assert_equal [File.join(@dir, "d.yml"), File.join(@dir, "e.yml")], file_paths
  end

  test "get on non-array root returns scalar" do
    result = Yerba::Collection.get(File.join(@dir, "[ab].yml"), "port")

    assert_equal 2, result.length
    assert(result.all?(Yerba::Scalar))

    values = result.map(&:value).sort
    assert_equal [1000, 2000], values
  end

  test "get returns empty array for non-matching selector" do
    result = Yerba::Collection.get(File.join(@dir, "a.yml"), "nonexistent")

    assert_equal [], result
  end

  test "get returns empty array for non-matching glob" do
    result = Yerba::Collection.get(File.join(@dir, "zzz*.yml"), "name")

    assert_equal [], result
  end

  test "get with wildcard returns flat results from all files" do
    collection = Yerba.files(File.join(@dir, "c.yml"))
    result = collection.get("items[].name")

    values = result.map(&:value)
    assert_equal ["One", "Two"], values
  end

  test "find returns items across files" do
    collection = Yerba.files(File.join(@dir, "c.yml"))
    result = collection.find("items[]")

    assert_equal 2, result.length
    assert_equal "One", result[0]["name"]
    assert_equal "Two", result[1]["name"]
  end

  test "find with condition filters across files" do
    File.write(File.join(@dir, "d.yml"), "- kind: talk\n  id: a\n- kind: keynote\n  id: b\n")
    collection = Yerba.files(File.join(@dir, "d.yml"))
    result = collection.find("[]", condition: '.kind == "keynote"')

    assert_equal 1, result.length
    assert_equal "b", result[0]["id"]
  end

  test "find with select returns only specified fields plus metadata" do
    File.write(File.join(@dir, "e.yml"), "- id: x\n  title: Hello\n  year: 2020\n- id: y\n  title: World\n  year: 2021\n")
    collection = Yerba.files(File.join(@dir, "e.yml"))
    result = collection.find("[]", select: "id,title")

    assert_equal 2, result.length
    assert_equal "x", result[0]["id"]
    assert_equal "Hello", result[0]["title"]
    refute result[0].key?("year")
    assert result[0].key?("__file")
  end

  test "find_by searches across files" do
    File.write(File.join(@dir, "d.yml"), <<~YAML)
      - id: talk-1
        name: Hello
      - id: talk-2
        name: World
    YAML
    File.write(File.join(@dir, "e.yml"), <<~YAML)
      - id: talk-3
        name: Foo
    YAML

    collection = Yerba.files(File.join(@dir, "[de].yml"))
    result = collection.find_by(name: "World")

    assert_instance_of Yerba::Map, result
    assert_equal "talk-2", result["id"].value
  end

  test "find_by returns nil when no match" do
    File.write(File.join(@dir, "d.yml"), <<~YAML)
      - id: talk-1
        name: Hello
    YAML

    collection = Yerba.files(File.join(@dir, "d.yml"))
    result = collection.find_by(name: "Missing")

    assert_nil result
  end

  test "where searches across files" do
    File.write(File.join(@dir, "d.yml"), <<~YAML)
      - id: talk-1
        kind: talk
      - id: talk-2
        kind: keynote
    YAML
    File.write(File.join(@dir, "e.yml"), <<~YAML)
      - id: talk-3
        kind: talk
    YAML

    collection = Yerba.files(File.join(@dir, "[de].yml"))
    results = collection.where(kind: "talk")

    assert_equal 2, results.length
    ids = results.map { |r| r["id"].value }
    assert_includes ids, "talk-1"
    assert_includes ids, "talk-3"
  end

  test "pluck collects values across files" do
    File.write(File.join(@dir, "d.yml"), <<~YAML)
      - id: talk-1
        name: Hello
      - id: talk-2
        name: World
    YAML
    File.write(File.join(@dir, "e.yml"), <<~YAML)
      - id: talk-3
        name: Foo
    YAML

    collection = Yerba.files(File.join(@dir, "[de].yml"))
    names = collection.pluck(:name)

    assert_equal 3, names.length
    assert_includes names, "Hello"
    assert_includes names, "World"
    assert_includes names, "Foo"
  end

  test "find_by skips non-sequence documents" do
    collection = Yerba.files(File.join(@dir, "a.yml"))
    result = collection.find_by(name: "Alpha")

    assert_nil result
  end

  test "apply yields each document and saves" do
    collection = Yerba.files(File.join(@dir, "[ab].yml"))

    collection.apply! do |document|
      document.set("port", 9999)
    end

    assert_includes File.read(File.join(@dir, "a.yml")), "port: 9999"
    assert_includes File.read(File.join(@dir, "b.yml")), "port: 9999"
  end
end
