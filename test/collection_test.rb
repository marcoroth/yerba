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

  test "get with non-array path returns array of values from each file" do
    collection = Yerba.files(File.join(@dir, "[ab].yml"))
    result = collection.get("name")

    assert_equal ["Alpha", "Beta"], result
  end

  test "get with array path returns flat array from all files" do
    collection = Yerba.files(File.join(@dir, "c.yml"))
    result = collection.get("items[].name")

    assert_equal ["One", "Two"], result
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

  test "apply yields each document and saves" do
    collection = Yerba.files(File.join(@dir, "[ab].yml"))

    collection.apply! do |document|
      document.set("port", 9999)
    end

    assert_includes File.read(File.join(@dir, "a.yml")), "port: 9999"
    assert_includes File.read(File.join(@dir, "b.yml")), "port: 9999"
  end
end
