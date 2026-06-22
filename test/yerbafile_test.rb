# frozen_string_literal: true

require "test_helper"
require "tempfile"
require "fileutils"

class YerbafileTest < Minitest::Spec
  test "Yerbafile.locate finds Yerbafile in directory" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile"), "rules: []")

    assert_equal File.join(dir, "Yerbafile"), Yerba::Yerbafile.locate(dir)
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.locate finds Yerbafile.yml" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile.yml"), "rules: []")

    assert_equal File.join(dir, "Yerbafile.yml"), Yerba::Yerbafile.locate(dir)
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.locate finds .yerbafile" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, ".yerbafile"), "rules: []")

    assert_equal File.join(dir, ".yerbafile"), Yerba::Yerbafile.locate(dir)
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.locate walks up to parent directory" do
    dir = Dir.mktmpdir
    child = File.join(dir, "sub", "deep")
    FileUtils.mkdir_p(child)
    File.write(File.join(dir, "Yerbafile"), "rules: []")

    assert_equal File.join(dir, "Yerbafile"), Yerba::Yerbafile.locate(child)
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.locate returns nil when not found" do
    dir = Dir.mktmpdir

    assert_nil Yerba::Yerbafile.locate(dir)
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.find returns Yerbafile instance" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile"), "rules: []")

    yerbafile = Yerba::Yerbafile.find(dir)

    assert_instance_of Yerba::Yerbafile, yerbafile
    assert_equal File.join(dir, "Yerbafile"), yerbafile.path
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.find returns nil when not found" do
    dir = Dir.mktmpdir

    assert_nil Yerba::Yerbafile.find(dir)
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.find! raises when not found" do
    dir = Dir.mktmpdir

    assert_raises(Yerba::Error) do
      Yerba::Yerbafile.find!(dir)
    end
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.new raises for non-existent path" do
    assert_raises(Yerba::Error) do
      Yerba::Yerbafile.new("/nonexistent/Yerbafile")
    end
  end

  test "Yerbafile#apply applies sort_keys rules to document" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile"), <<~YAML)
      rules:
        - files: "**/*.yml"
          pipeline:
            - sort_keys:
                path: "[]"
                order:
                  - name
                  - slug
                  - github
    YAML

    yerbafile = Yerba::Yerbafile.find!(dir)

    document = Yerba::Document.parse(<<~YAML)
      - github: aalice
        name: Alice
        slug: alice
    YAML

    yerbafile.apply(document)

    assert_equal <<~YAML, document.to_s
      - name: Alice
        slug: alice
        github: aalice
    YAML
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile#apply applies quote_style rules to document" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile"), <<~YAML)
      rules:
        - files: "**/*.yml"
          pipeline:
            - quote_style:
                key_style: plain
                value_style: double
    YAML

    yerbafile = Yerba::Yerbafile.find!(dir)

    document = Yerba::Document.parse(<<~YAML)
      name: Alice
      role: admin
    YAML

    yerbafile.apply(document)

    assert_equal <<~YAML, document.to_s
      name: "Alice"
      role: "admin"
    YAML
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile#apply applies multiple pipeline steps in order" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile"), <<~YAML)
      rules:
        - files: "**/*.yml"
          pipeline:
            - quote_style:
                key_style: plain
                value_style: double
            - sort_keys:
                path: "[]"
                order:
                  - name
                  - slug
    YAML

    yerbafile = Yerba::Yerbafile.find!(dir)

    document = Yerba::Document.parse(<<~YAML)
      - slug: alice
        name: Alice
    YAML

    yerbafile.apply(document)

    assert_equal <<~YAML, document.to_s
      - name: "Alice"
        slug: "alice"
    YAML
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.resolve with nil finds Yerbafile" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile"), "rules: []")

    Dir.chdir(dir) do
      yerbafile = Yerba::Yerbafile.resolve(nil)

      assert_instance_of Yerba::Yerbafile, yerbafile
    end
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.resolve with string path creates instance" do
    dir = Dir.mktmpdir
    path = File.join(dir, "Yerbafile")
    File.write(path, "rules: []")

    yerbafile = Yerba::Yerbafile.resolve(path)

    assert_instance_of Yerba::Yerbafile, yerbafile
    assert_equal path, yerbafile.path
  ensure
    FileUtils.rm_rf(dir)
  end

  test "apply_yerbafile works with absolute file paths" do
    dir = Dir.mktmpdir
    data_dir = File.join(dir, "data")
    FileUtils.mkdir_p(data_dir)

    File.write(File.join(dir, "Yerbafile"), <<~YAML)
      files: "data/**/*.yml"

      pipeline:
        - sequence_indent:
            style: indented

        - quote_style:
            key_style: plain
            value_style: double

      rules:
        - files: "data/speakers.yml"
          pipeline:
            - sort_keys:
                path: "[]"
                order:
                  - name
                  - github
                  - slug
    YAML

    yaml_path = File.join(data_dir, "speakers.yml")
    File.write(yaml_path, <<~YAML)
      - slug: alice
        name: Alice
        github: alice
    YAML

    document = Yerba.parse_file(yaml_path)
    yerbafile_path = File.join(dir, "Yerbafile")

    document.apply_yerbafile(yerbafile_path)

    assert document.changed?
    assert_equal <<~YAML, document.to_s
      - name: "Alice"
        github: "alice"
        slug: "alice"
    YAML
  ensure
    FileUtils.rm_rf(dir)
  end

  test "Yerbafile.resolve with Yerbafile instance passes through" do
    dir = Dir.mktmpdir
    File.write(File.join(dir, "Yerbafile"), "rules: []")

    original = Yerba::Yerbafile.find!(dir)
    resolved = Yerba::Yerbafile.resolve(original)

    assert_same original, resolved
  ensure
    FileUtils.rm_rf(dir)
  end
end
