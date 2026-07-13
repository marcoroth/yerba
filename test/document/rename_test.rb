# frozen_string_literal: true

require "test_helper"

module Document
  class RenameTest < Minitest::Spec
    test "rename renames a root key in place" do
      document = Yerba::Document.parse(<<~YAML)
        host: localhost
        port: 5432
      YAML

      document.rename("host", "hostname")

      assert_equal <<~YAML, document.to_s
        hostname: localhost
        port: 5432
      YAML
    end

    test "rename renames a nested key in place when the destination is a full path" do
      document = Yerba::Document.parse(<<~YAML)
        database:
          host: localhost
          port: 5432
      YAML

      document.rename("database.host", "database.hostname")

      assert_equal <<~YAML, document.to_s
        database:
          hostname: localhost
          port: 5432
      YAML
    end

    test "rename renames a key inside a sequence item in place" do
      document = Yerba::Document.parse(<<~YAML)
        items:
          - name: Ruby
            year: 1995
      YAML

      document.rename("items[0].year", "items[0].founded")

      assert_equal <<~YAML, document.to_s
        items:
          - name: Ruby
            founded: 1995
      YAML
    end

    test "rename moves a nested key to the document root when the destination is a bare name" do
      document = Yerba::Document.parse(<<~YAML)
        database:
          host: localhost
          port: 5432
      YAML

      document.rename("database.host", "hostname")

      assert_equal <<~YAML, document.to_s
        database:
          port: 5432
        hostname: localhost
      YAML
    end

    test "rename moves a key to another map" do
      document = Yerba::Document.parse(<<~YAML)
        database:
          host: localhost
          port: 5432
        settings:
          debug: true
      YAML

      document.rename("database.host", "settings.db_host")

      assert_equal <<~YAML, document.to_s
        database:
          port: 5432
        settings:
          debug: true
          db_host: localhost
      YAML
    end

    test "rename moves a root key into a nested map" do
      document = Yerba::Document.parse(<<~YAML)
        hostname: localhost
        database:
          port: 5432
      YAML

      document.rename("hostname", "database.host")

      assert_equal <<~YAML, document.to_s
        database:
          port: 5432
          host: localhost
      YAML
    end

    test "rename raises for a missing selector" do
      document = Yerba::Document.parse("host: localhost\n")

      assert_raises(Yerba::Error) do
        document.rename("missing", "new_name")
      end
    end
  end
end
