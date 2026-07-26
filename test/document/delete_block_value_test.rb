# frozen_string_literal: true

require "test_helper"

class DocumentDeleteBlockValueTest < Minitest::Spec
  test "deletes a block sequence entry at the end of a map" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        id: "a"
        tags:
          - old
    YAML

    document.delete("conf.tags")

    assert_equal <<~YAML, document.to_s
      conf:
        id: "a"
    YAML
  end

  test "deletes a block sequence entry at the start of a map" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        tags:
          - old
        name: "N"
    YAML

    document.delete("conf.tags")

    assert_equal <<~YAML, document.to_s
      conf:
        name: "N"
    YAML
  end

  test "deletes a block sequence entry in the middle of a map" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        id: "a"
        tags:
          - old
        name: "N"
    YAML

    document.delete("conf.tags")

    assert_equal <<~YAML, document.to_s
      conf:
        id: "a"
        name: "N"
    YAML
  end

  test "deletes a block map entry in the middle of a map" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        id: "a"
        db:
          host: h
        name: "N"
    YAML

    document.delete("conf.db")

    assert_equal <<~YAML, document.to_s
      conf:
        id: "a"
        name: "N"
    YAML
  end

  test "deletes a block sequence entry in the middle of the document root" do
    document = Yerba::Document.parse(<<~YAML)
      id: "a"
      tags:
        - old
      name: "N"
    YAML

    document.delete("tags")

    assert_equal <<~YAML, document.to_s
      id: "a"
      name: "N"
    YAML
  end

  test "keeps a comment that follows the deleted entry" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        tags:
          - old
        # note
        name: "N"
    YAML

    document.delete("conf.tags")

    assert_equal <<~YAML, document.to_s
      conf:
        # note
        name: "N"
    YAML
  end

  test "keeps the indentation of the following entry across a blank line" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        id: "a"

        tags:
          - old
        name: "N"
    YAML

    document.delete("conf.tags")

    assert_equal <<~YAML, document.to_s
      conf:
        id: "a"

        name: "N"
    YAML
  end

  test "deletes the only entry of a map" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        tags:
          - old
    YAML

    document.delete("conf.tags")

    assert_equal "conf: {}\n", document.to_s
  end

  test "replaces a block value in the middle of a map" do
    document = Yerba::Document.parse(<<~YAML)
      conf:
        id: "a"
        tags:
          - old
        name: "N"
    YAML

    document["conf"]["tags"] = ["new", "fresh"]

    assert_equal <<~YAML, document.to_s
      conf:
        id: "a"
        tags:
          - new
          - fresh
        name: "N"
    YAML
  end
end
