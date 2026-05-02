# frozen_string_literal: true

require "test_helper"

class ConditionTest < Minitest::Spec
  test "condition? returns true when condition matches" do
    document = Yerba::Document.parse("kind: keynote\ntitle: Opening")

    assert document.condition?('.kind == "keynote"')
  end

  test "condition? returns false when condition does not match" do
    document = Yerba::Document.parse("kind: keynote\ntitle: Opening")

    refute document.condition?('.kind == "talk"')
  end

  test "condition? supports != operator" do
    document = Yerba::Document.parse("kind: keynote")

    assert document.condition?('.kind != "talk"')
  end

  test "condition? supports contains operator" do
    document = Yerba::Document.parse("title: Ruby on Rails")

    assert document.condition?('.title contains "Ruby"')
  end

  test "condition? with path scopes to parent" do
    document = Yerba::Document.parse("database:\n  host: localhost\n  port: 5432")

    assert document.condition?('.host == "localhost"', path: "database")
  end

  test "delete with condition removes key when condition matches" do
    document = Yerba::Document.parse("kind: keynote\ndate_precision: day")
    document.delete("date_precision", condition: '.date_precision == "day"')

    refute_includes document.to_s, "date_precision"
  end

  test "delete with condition keeps key when condition does not match" do
    document = Yerba::Document.parse("kind: keynote\ndate_precision: month")
    document.delete("date_precision", condition: '.date_precision == "day"')

    assert_includes document.to_s, "date_precision: month"
  end

  test "set with condition updates when condition matches" do
    document = Yerba::Document.parse("status: draft\ntitle: Hello")
    document.set("title", "Updated", condition: '.status == "draft"')

    assert_includes document.to_s, "title: Updated"
  end

  test "set with condition skips when condition does not match" do
    document = Yerba::Document.parse("status: published\ntitle: Hello")
    document.set("title", "Updated", condition: '.status == "draft"')

    assert_includes document.to_s, "title: Hello"
  end

  test "set with if_exists updates existing key" do
    document = Yerba::Document.parse("name: Alice\nage: 30")
    document.set("age", 31, if_exists: true)

    assert_includes document.to_s, "age: 31"
  end

  test "set with if_exists skips missing key" do
    document = Yerba::Document.parse("name: Alice")
    document.set("age", 31, if_exists: true)

    refute_includes document.to_s, "age"
  end

  test "set with if_missing skips when key exists" do
    document = Yerba::Document.parse("name: Alice\nage: 30")
    document.set("age", 99, if_missing: true)

    assert_includes document.to_s, "age: 30"
  end

  test "set with if_missing raises when key is absent" do
    document = Yerba::Document.parse("name: Alice")

    assert_raises(Yerba::Error) do
      document.set("age", 30, if_missing: true)
    end
  end
end
