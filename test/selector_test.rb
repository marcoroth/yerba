# frozen_string_literal: true

require "test_helper"

class SelectorTest < Minitest::Spec
  test "parses keys and indexes into segments" do
    selector = Yerba::Selector.parse("[3].talks[1]")

    assert_equal [3, "talks", 1], selector.segments
    assert selector.absolute?
    refute selector.relative?
    refute selector.wildcard?
  end

  test "parses nested keys" do
    assert_equal ["database", "host"], Yerba::Selector.parse("database.host").segments
  end

  test "parses wildcards" do
    assert_equal [:all_items, "speakers", :all_items], Yerba::Selector.parse("[].speakers[]").segments
    assert_equal ["database", :all_keys], Yerba::Selector.parse("database.*").segments

    assert Yerba::Selector.parse("[].title").wildcard?
    assert Yerba::Selector.parse("database.*").wildcard?
  end

  test "parses relative selectors" do
    selector = Yerba::Selector.parse(".speakers[].name")

    assert selector.relative?
    assert_equal ["speakers", :all_items, "name"], selector.segments
  end

  test "parses the empty selector as the document root" do
    selector = Yerba::Selector.parse("")

    assert selector.empty?
    assert_equal [], selector.segments
    assert_equal "", selector.to_s
  end

  test "caches parsed selectors" do
    assert_same Yerba::Selector.parse("[0].talks[2]"), Yerba::Selector.parse("[0].talks[2]")
  end

  test "parse returns a selector untouched" do
    selector = Yerba::Selector.parse("[0]")

    assert_same selector, Yerba::Selector.parse(selector)
  end

  test "builds selectors from segments" do
    assert_equal "[3].talks[1]", Yerba::Selector.build([3, "talks", 1]).to_s
    assert_equal "database.host", Yerba::Selector.build(["database", "host"]).to_s
    assert_equal "[].speakers[]", Yerba::Selector.build([:all_items, "speakers", :all_items]).to_s
    assert_equal "database.*", Yerba::Selector.build(["database", :all_keys]).to_s
    assert_equal ".title", Yerba::Selector.build(["title"], relative: true).to_s
    assert_equal "", Yerba::Selector.build([]).to_s
  end

  test "round-trips through parse and build" do
    ["[3].talks[1]", "database.host", "[].speakers[].name", "database.*", "[0]", "[]"].each do |string|
      assert_equal string, Yerba::Selector.build(Yerba::Selector.parse(string).segments).to_s
    end
  end

  test "parent and join navigate the selector" do
    selector = Yerba::Selector.parse("[3].talks[1]")

    assert_equal "[3].talks", selector.parent.to_s
    assert_equal "[3].talks[1].speakers[0]", selector.join("speakers", 0).to_s
    assert_equal selector, Yerba::Selector.parse("[3]").join("talks", 1)
  end

  test "dig looks up plain values like value_at does" do
    document = Yerba.parse(<<~YAML)
      - id: lightning-talks
        talks:
          - id: lightning-one
            title: Lightning One
          - id: lightning-two
            title: Lightning Two
    YAML

    ["[0]", "[0].talks", "[0].talks[1]", "[0].talks[1].title", "[0].id"].each do |string|
      assert_equal document.value_at(string), Yerba::Selector.parse(string).dig(document.values), string
    end
  end

  test "dig returns nil for paths that don't exist" do
    values = Yerba.parse("- id: a\n").values

    assert_nil Yerba::Selector.parse("[5].talks[0]").dig(values)
    assert_nil Yerba::Selector.parse("[0].id.nested").dig(values)
    assert_nil Yerba::Selector.parse("[0][1]").dig(values)
  end

  test "dig raises for wildcard selectors" do
    assert_raises(ArgumentError) { Yerba::Selector.parse("[].id").dig([]) }
  end

  test "selectors are equal by segments" do
    assert_equal Yerba::Selector.parse("[1].title"), Yerba::Selector.build([1, "title"])
    refute_equal Yerba::Selector.parse(".title"), Yerba::Selector.parse("title")
    assert_equal 1, [Yerba::Selector.parse("[1]"), Yerba::Selector.build([1])].uniq.size
  end
end
