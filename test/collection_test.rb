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

  test "apply yields each document and saves" do
    collection = Yerba.files(File.join(@dir, "[ab].yml"))

    collection.apply! do |document|
      document.set("port", 9999)
    end

    assert_includes File.read(File.join(@dir, "a.yml")), "port: 9999"
    assert_includes File.read(File.join(@dir, "b.yml")), "port: 9999"
  end
end
