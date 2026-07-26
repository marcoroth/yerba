# frozen_string_literal: true

require "test_helper"

class DocumentWildcardTest < Minitest::Spec
  YAML_SOURCE = <<~YAML
    name: "Conf"
    venue:
      city: "Berlin"
      country: "DE"
    items:
      - name: "a"
      - name: "b"
  YAML

  def document
    @document ||= Yerba::Document.parse(YAML_SOURCE)
  end

  test "get_all resolves both wildcards" do
    assert_equal ["Berlin", "DE"], document.get_all("venue.*").map(&:value)
    assert_equal ["a", "b"], document.get_all("items[].name").map(&:value)
  end

  test "indexing resolves both wildcards" do
    assert_equal ["Berlin", "DE"], document["venue.*"].map(&:value)
    assert_equal ["a", "b"], document["items[].name"].map(&:value)
  end

  test "value_at resolves both wildcards" do
    assert_equal ["Berlin", "DE"], document.value_at("venue.*")
    assert_equal ["a", "b"], document.value_at("items[].name")
  end

  test "find resolves both wildcards" do
    assert_equal ["Berlin", "DE"], document.find("venue.*")
    assert_equal ["a", "b"], document.find("items[].name")
  end

  test "exists? accepts both wildcards" do
    assert document.exists?("venue.*")
    assert document.exists?("items[].name")
  end

  test "exists? rejects a wildcard that matches nothing" do
    refute document.exists?("venue.missing.*")
    refute document.exists?("items[].missing")
  end

  test "valid_selector? accepts both wildcards" do
    assert document.valid_selector?("venue.*")
    assert document.valid_selector?("items[]")
  end

  test "every read path agrees on what a wildcard resolves to" do
    ["venue.*", "items[].name", "*.city"].each do |selector|
      values = document.get_all(selector).map(&:value)

      assert_equal values, Array(document[selector]).map(&:value), "document[#{selector.inspect}]"
      assert_equal values, Array(document.value_at(selector)).compact, "value_at(#{selector.inspect})"
    end
  end

  test "value_at keeps a positional nil for a branch that does not match" do
    assert_equal [nil, "Berlin", nil], document.value_at("*.city")
    assert_equal ["Berlin"], document.get_all("*.city").map(&:value)

    assert_equal [nil, nil], document.value_at("items[].city")
    assert_empty document.get_all("items[].city")
  end

  test "* resolves the values of the document root" do
    assert_equal(["name", "venue", "items"], document.get_all("*").map { |node| node.key.value })
    assert_equal "Conf", document.get_all("*").first.value
  end

  test "* keeps each value's own type" do
    assert_equal [Yerba::Scalar, Yerba::Map, Yerba::Sequence], document.get_all("*").map(&:class)
  end

  test "* combines with []" do
    assert_equal ["a", "b"], document.get_all("items[].*").map(&:value)
  end

  test "a key can follow *" do
    assert_equal ["Berlin"], document.get_all("*.city").map(&:value)
  end

  test "* returns nodes that are still connected to the document" do
    document.get_all("venue.*").first.value = "Hamburg"

    assert_equal "Hamburg", document.value_at("venue.city")
  end

  test "deleting through * is refused rather than removing every value" do
    assert_raises(Yerba::Error) { document.delete("venue.*") }

    assert_equal({ "city" => "Berlin", "country" => "DE" }, document.value_at("venue"))
  end

  test "setting through * is refused rather than writing every value" do
    assert_raises(Yerba::Error) { document.set("venue.*", "x") }

    assert_equal({ "city" => "Berlin", "country" => "DE" }, document.value_at("venue"))
  end

  test "inserting through * is refused rather than creating a key named *" do
    assert_raises(Yerba::Error) { document.insert("venue.*", "x") }

    assert_equal ["city", "country"], document.keys_at("venue")
    refute_includes document.to_s, "*"
  end

  test "a key containing * is treated as a key rather than a wildcard" do
    document = Yerba::Document.parse(<<~YAML)
      a*b: 1
      name: Conf
    YAML

    assert_instance_of Yerba::Scalar, document["a*b"]
    assert_equal 1, document["a*b"].value
    refute document.valid_selector?("*.missing")
  end

  test "a * select field stays an array when it matches a single value" do
    document = Yerba::Document.parse(<<~YAML)
      items:
        - venue:
            city: "Berlin"
        - venue:
            city: "Hamburg"
            country: "DE"
    YAML

    assert_equal [{ "venue" => ["Berlin"] }, { "venue" => ["Hamburg", "DE"] }], document.find("items[]", select: "venue.*")
  end

  test "* resolves the values of a nested map" do
    nested = Yerba::Document.parse(<<~YAML)
      a:
        b:
          c: 1
          d: 2
    YAML

    assert_equal [1, 2], nested.get_all("a.b.*").map(&:value)
    assert_equal ["a.b.c", "a.b.d"], nested.get_all("a.b.*").map(&:selector)
  end

  test "* chains with itself" do
    nested = Yerba::Document.parse(<<~YAML)
      a:
        b:
          c: 1
          d: 2
    YAML

    assert_equal [1, 2], nested.get_all("a.*.*").map(&:value)
  end

  test "* resolves nothing for an empty map" do
    empty = Yerba::Document.parse("db: {}\n")

    assert_empty empty.get_all("db.*")
    refute empty.exists?("db.*")
  end

  test "* does not resolve the values of a flow map" do
    flow = Yerba::Document.parse("db: {host: localhost, port: 5432}\n")

    assert_equal ["host", "port"], flow["db"].keys
    assert_nil flow["db.host"]
    assert_empty flow.get_all("db.*")
  end

  test "* resolves across a glob" do
    directory = Dir.mktmpdir

    File.write(File.join(directory, "one.yml"), "city: \"Berlin\"\ncountry: \"DE\"\n")
    File.write(File.join(directory, "two.yml"), "city: \"Lisbon\"\n")

    nodes = Yerba::Collection.get(File.join(directory, "*.yml"), "*")

    assert_equal ["Berlin", "DE", "Lisbon"], nodes.map(&:value)
    assert_equal(["city", "country", "city"], nodes.map { |node| node.key.value })
  ensure
    FileUtils.rm_rf(directory)
  end
end
