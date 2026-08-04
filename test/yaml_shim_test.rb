# frozen_string_literal: true

require "test_helper"
require "yerba/yaml"
require "tmpdir"
require "fileutils"

class YAMLShimTest < Minitest::Spec
  SOURCE = <<~YAML
    # Database settings
    database:
      host: localhost   # dev only
      pool: 5
    tags: [ruby, rails]
  YAML

  EXPECTED = {
    "database" => { "host" => "localhost", "pool" => 5 },
    "tags" => ["ruby", "rails"],
  }.freeze

  def with_file
    dir = Dir.mktmpdir
    path = File.join(dir, "config.yml")
    File.write(path, SOURCE)

    yield path, dir
  ensure
    FileUtils.rm_rf(dir)
  end

  after do
    Yerba::YAML.takeover!
  end

  test "the shim adds no methods to ::YAML" do
    added = YAML.singleton_class.ancestors
                .take_while { |mod| mod != Psych.singleton_class }
                .flat_map { |mod| mod.instance_methods(false) }

    assert_equal [], added - Psych.singleton_methods
  end

  test "it is active after require" do
    assert Yerba::YAML.active?
  end

  test "load and load_file are backed by Yerba" do
    with_file do |path|
      assert_equal EXPECTED, YAML.load(SOURCE)
      assert_equal EXPECTED, YAML.load_file(path)
    end
  end

  test "the safe and unsafe loaders are backed by Yerba" do
    with_file do |path|
      assert_equal EXPECTED, YAML.safe_load(SOURCE)
      assert_equal EXPECTED, YAML.unsafe_load(SOURCE)
      assert_equal EXPECTED, YAML.safe_load_file(path)
      assert_equal EXPECTED, YAML.unsafe_load_file(path)
    end
  end

  test "restore! hands the loaders back to Psych, takeover! takes them again" do
    source = "when: 2020-01-01\n"

    assert_equal "2020-01-01", YAML.load(source, permitted_classes: [Date])["when"]

    Yerba::YAML.restore!

    refute Yerba::YAML.active?
    assert_equal Date.new(2020, 1, 1), YAML.load(source, permitted_classes: [Date])["when"]

    Yerba::YAML.takeover!

    assert_equal "2020-01-01", YAML.load(source, permitted_classes: [Date])["when"]
  end

  test "takeover! with a block is scoped to the block and returns its value" do
    Yerba::YAML.restore!

    result = Yerba::YAML.takeover! do
      assert Yerba::YAML.active?

      YAML.load(SOURCE)
    end

    assert_equal EXPECTED, result
    refute Yerba::YAML.active?
  end

  test "restore! with a block hands the loaders back just for the block" do
    source = "when: 2020-01-01\n"

    result = Yerba::YAML.restore! do
      refute Yerba::YAML.active?

      YAML.load(source, permitted_classes: [Date])["when"]
    end

    assert_equal Date.new(2020, 1, 1), result
    assert Yerba::YAML.active?
    assert_equal "2020-01-01", YAML.load(source, permitted_classes: [Date])["when"]
  end

  test "the block forms nest" do
    Yerba::YAML.takeover! do
      Yerba::YAML.restore! do
        Yerba::YAML.takeover! { assert Yerba::YAML.active? }

        refute Yerba::YAML.active?
      end

      assert Yerba::YAML.active?
    end
  end

  test "the block form restores state when the block raises" do
    assert_raises(RuntimeError) { Yerba::YAML.restore! { raise "boom" } }

    assert Yerba::YAML.active?
  end

  test "the block form does not affect other threads" do
    Yerba::YAML.restore! do
      assert Thread.new { Yerba::YAML.active? }.value
    end
  end

  test "symbolize_names deep-symbolizes keys" do
    assert_equal({ database: { host: "localhost", pool: 5 }, tags: ["ruby", "rails"] }, YAML.load(SOURCE, symbolize_names: true))
  end

  test "freeze deep-freezes the result" do
    result = YAML.load(SOURCE, freeze: true)

    assert_predicate result, :frozen?
    assert_predicate result["database"], :frozen?
    assert_predicate result["database"]["host"], :frozen?
    assert_predicate result["tags"], :frozen?
    assert_predicate result["tags"].first, :frozen?
  end

  test "fallback matches Psych: nil for load, false for unsafe_load" do
    assert_nil YAML.load("")
    assert_equal false, YAML.unsafe_load("")
    assert_equal :none, YAML.load("", fallback: :none)
    assert_equal :none, YAML.safe_load("", fallback: :none)
  end

  test "Psych-only options are accepted and ignored" do
    assert_equal EXPECTED, YAML.load(SOURCE, permitted_classes: [Date], aliases: true, strict_integer: true)
    assert_equal EXPECTED, YAML.safe_load(SOURCE, permitted_symbols: [:a], filename: "config.yml")
  end

  test "parse errors are raised as Psych::SyntaxError with file and line" do
    error = assert_raises(Psych::SyntaxError) { YAML.load("a:\n- b\n  c: d\n", filename: "broken.yml") }

    assert_equal "broken.yml", error.file
    assert_operator error.line, :>, 0
  end

  test "load_file reports the path on a parse error" do
    with_file do |path|
      File.write(path, "a:\n- b\n  c: d\n")

      error = assert_raises(Psych::SyntaxError) { YAML.load_file(path) }

      assert_equal path, error.file
    end
  end

  test "dump is left on Psych, so newlines are not corrupted" do
    value = { "k" => "multi\nline" }

    assert_equal value, YAML.load(YAML.dump(value))
  end

  test "documented divergences from Psych" do
    aliased = "base: &b\n  a: 1\nmerged:\n  <<: *b\nref: *b\n"

    assert_equal({ "<<" => "*b" }, YAML.load(aliased)["merged"])
    assert_equal "*b", YAML.load(aliased)["ref"]
    assert_equal({ "a" => 1 }, YAML.load("a: 1\n---\nb: 2\n"))
    assert_equal "2020-01-01 10:00:00", YAML.load("at: 2020-01-01 10:00:00\n")["at"]
  end

  test "NOT_REPLACED methods still come from Psych" do
    Yerba::YAML::NOT_REPLACED.each do |name|
      assert_equal Psych.singleton_class, YAML.method(name).owner, "expected ::YAML.#{name} to still be Psych's"
    end
  end
end
