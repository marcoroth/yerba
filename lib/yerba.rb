# frozen_string_literal: true

require "fileutils"

require_relative "yerba/version"
require_relative "yerba/location"
require_relative "yerba/selector"
require_relative "yerba/formatting"
require_relative "yerba/node"
require_relative "yerba/scalar"
require_relative "yerba/map"
require_relative "yerba/sequence"
require_relative "yerba/query_result"
require_relative "yerba/document"
require_relative "yerba/collection"
require_relative "yerba/yerbafile"

begin
  major, minor, = RUBY_VERSION.split(".")
  require "yerba/#{major}.#{minor}/yerba"
rescue LoadError
  begin
    require "yerba/yerba"
  rescue LoadError
    # C extension not available, fall back to CLI mode
  end
end

module Yerba
  # @rbs!
  #   # A plain Ruby value, as returned by Document#value_at, Document#values, Document#to_h, ...
  #   type scalar_value = String | Integer | Float | bool | nil
  #
  #   type value = scalar_value | Array[value] | Hash[String, value]
  #
  #   # A node of a document, as returned by Document#[]
  #   type node = Scalar | Map | Sequence
  #
  #   # Returned by Document#get_quote_style and Scalar#quote_style
  #   type quote_style = :plain | :single | :double | :literal | :literal_clip | :literal_keep | :folded | :folded_clip | :folded_keep
  #
  #   # Returned by Document#get_collection_style, Map#collection_style and Sequence#collection_style
  #   type collection_style = :block | :flow
  #
  #   # Returned by Document#get_sequence_indent and Sequence#sequence_indent
  #   type sequence_indent = :compact | :indented
  #
  #   # Defined in the C extension (module function).
  #   def self?.quote_scalar: (String value, ?(quote_style | String)? style) -> String
  #
  #   # Defined in the C extension.
  #   class Error < StandardError
  #   end
  #
  #   class PathNotFoundError < Error
  #   end
  #
  #   class ParseError < Error
  #   end
  #
  #   class PathValidationError < Error
  #   end

  class UnsupportedPlatformError < StandardError; end
  class ExecutableNotFoundError < StandardError; end
  class CompilationError < StandardError; end
  class SelectorNotFoundError < StandardError; end
  class StaleFileError < StandardError; end

  GEM_NAME = "yerba" #: String
  EXECUTABLE_NAME = "yerba" #: String

  NATIVE_PLATFORMS = {
    "arm64-darwin" => "arm64-darwin",
    "x86_64-darwin" => "x86_64-darwin",
    "aarch64-linux" => "aarch64-linux-gnu",
    "arm64-linux" => "aarch64-linux-gnu",
    "x86_64-linux" => "x86_64-linux-gnu",
  }.freeze #: Hash[String, String]

  #: (?exe_path: String?) -> String
  def self.executable(exe_path: nil)
    if exe_path
      return exe_path if File.executable?(exe_path)

      raise ExecutableNotFoundError, "yerba executable not found at #{exe_path}"
    end

    install_dir = ENV.fetch("YERBA_INSTALL_DIR", nil)

    if install_dir
      install_dir_exe = File.join(install_dir, EXECUTABLE_NAME)

      if File.executable?(install_dir_exe)
        return install_dir_exe
      end

      raise ExecutableNotFoundError, "yerba executable not found at #{install_dir_exe} (set by YERBA_INSTALL_DIR)"
    end

    platform = Gem::Platform.local
    platform_key = "#{platform.cpu}-#{platform.os}"

    exe_directory = NATIVE_PLATFORMS[platform_key]

    if exe_directory
      exe_file = File.expand_path(
        File.join("..", "exe", exe_directory, EXECUTABLE_NAME),
        __dir__ #: String
      )

      return exe_file if File.executable?(exe_file)
    end

    compiled = compile_from_source

    return compiled if compiled

    if exe_directory
      raise ExecutableNotFoundError,
            "yerba executable not found at exe/#{exe_directory}/yerba. " \
            "Try reinstalling the gem: gem install yerba"
    else
      raise UnsupportedPlatformError,
            "yerba does not have a precompiled binary for #{platform_key}. " \
            "Install Rust (https://rustup.rs) and reinstall the gem to compile from source, " \
            "or set YERBA_INSTALL_DIR to use a custom binary."
    end
  end

  #: () -> String?
  def self.compile_from_source
    lib_dir = __dir__ #: String
    rust_dir = File.expand_path(File.join("..", "rust"), lib_dir)

    return nil unless File.exist?(File.join(rust_dir, "Cargo.toml"))
    return nil unless system("cargo --version > /dev/null 2>&1")

    platform = Gem::Platform.local
    platform_key = "#{platform.cpu}-#{platform.os}"
    exe_directory = File.join(File.expand_path(File.join("..", "exe", platform_key), lib_dir))
    exe_file = File.join(exe_directory, EXECUTABLE_NAME)

    return exe_file if File.executable?(exe_file)

    warn "yerba: No precompiled binary found. Compiling from source..."

    FileUtils.mkdir_p(exe_directory)

    root_dir = File.expand_path(File.join("..", ".."), lib_dir)
    workspace_target = File.join(root_dir, "target", "release", EXECUTABLE_NAME)
    crate_target = File.join(rust_dir, "target", "release", EXECUTABLE_NAME)

    unless system("cd #{root_dir} && cargo build --release")
      raise CompilationError, "Failed to compile yerba from source. Is Rust installed?"
    end

    source_binary = [workspace_target, crate_target].find { |p| File.exist?(p) }

    unless source_binary
      raise CompilationError, "Compilation succeeded but binary not found at #{workspace_target} or #{crate_target}"
    end

    FileUtils.cp(source_binary, exe_file)
    FileUtils.chmod(0o755, exe_file)

    exe_file
  end

  private_class_method :compile_from_source

  #: (String content) -> Document
  def self.parse(content)
    Document.parse(content)
  end

  #: (String path) -> Document
  def self.parse_file(path)
    Document.new(path)
  end

  #: (String path) -> Document
  def self.document(path)
    Document.new(path)
  end

  #: (String glob) -> Collection
  def self.files(glob)
    Collection.new(glob)
  end
end
