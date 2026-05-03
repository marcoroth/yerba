# frozen_string_literal: true

require "test_helper"

class ConditionTest < Minitest::Spec
  test "condition? returns true when condition matches" do
    document = Yerba::Document.parse(<<~YAML)
      kind: keynote
      title: Opening
    YAML

    assert document.condition?('.kind == "keynote"')
  end

  test "condition? returns false when condition does not match" do
    document = Yerba::Document.parse(<<~YAML)
      kind: keynote
      title: Opening
    YAML

    refute document.condition?('.kind == "talk"')
  end

  test "condition? supports != operator" do
    document = Yerba::Document.parse(<<~YAML)
      kind: keynote
    YAML

    assert document.condition?('.kind != "talk"')
  end

  test "condition? supports contains operator" do
    document = Yerba::Document.parse(<<~YAML)
      title: Ruby on Rails
    YAML

    assert document.condition?('.title contains "Ruby"')
  end

  test "condition? with path scopes to parent" do
    document = Yerba::Document.parse(<<~YAML)
      database:
        host: localhost
        port: 5432
    YAML

    assert document.condition?('.host == "localhost"', path: "database")
  end

  test "delete with condition removes key when condition matches" do
    document = Yerba::Document.parse(<<~YAML)
      kind: keynote
      date_precision: day
    YAML
    document.delete("date_precision", condition: '.date_precision == "day"')

    refute_includes document.to_s, "date_precision"
  end

  test "delete with condition keeps key when condition does not match" do
    document = Yerba::Document.parse(<<~YAML)
      kind: keynote
      date_precision: month
    YAML
    document.delete("date_precision", condition: '.date_precision == "day"')

    assert_includes document.to_s, "date_precision: month"
  end

  test "set with condition updates when condition matches" do
    document = Yerba::Document.parse(<<~YAML)
      status: draft
      title: Hello
    YAML
    document.set("title", "Updated", condition: '.status == "draft"')

    assert_includes document.to_s, "title: Updated"
  end

  test "set with condition skips when condition does not match" do
    document = Yerba::Document.parse(<<~YAML)
      status: published
      title: Hello
    YAML
    document.set("title", "Updated", condition: '.status == "draft"')

    assert_includes document.to_s, "title: Hello"
  end

  test "set with if_exists updates existing key" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      age: 30
    YAML
    document.set("age", 31, if_exists: true)

    assert_includes document.to_s, "age: 31"
  end

  test "set with if_exists skips missing key" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML
    document.set("age", 31, if_exists: true)

    refute_includes document.to_s, "age"
  end

  test "set with if_missing skips when key exists" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      age: 30
    YAML
    document.set("age", 99, if_missing: true)

    assert_includes document.to_s, "age: 30"
  end

  test "set with if_missing raises when key is absent" do
    document = Yerba::Document.parse(<<~YAML)
      name: Alice
    YAML

    assert_raises(Yerba::Error) do
      document.set("age", 30, if_missing: true)
    end
  end
end
