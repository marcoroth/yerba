# frozen_string_literal: true

require "fileutils"

require_relative "yerba/version"
require_relative "yerba/location"
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
  class UnsupportedPlatformError < StandardError; end
  class ExecutableNotFoundError < StandardError; end
  class CompilationError < StandardError; end
  class SelectorNotFoundError < StandardError; end
  class StaleFileError < StandardError; end

  GEM_NAME = "yerba"
  EXECUTABLE_NAME = "yerba"

  NATIVE_PLATFORMS = {
    "arm64-darwin" => "arm64-darwin",
    "x86_64-darwin" => "x86_64-darwin",
    "aarch64-linux" => "aarch64-linux-gnu",
    "arm64-linux" => "aarch64-linux-gnu",
    "x86_64-linux" => "x86_64-linux-gnu",
  }.freeze

  def self.executable(exe_path: nil)
    if exe_path
      return exe_path if File.executable?(exe_path)

      raise ExecutableNotFoundError, "yerba executable not found at #{exe_path}"
    end

    if ENV["YERBA_INSTALL_DIR"]
      install_dir_exe = File.join(ENV["YERBA_INSTALL_DIR"], EXECUTABLE_NAME)

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
        __dir__
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

  def self.compile_from_source
    rust_dir = File.expand_path(File.join("..", "rust"), __dir__)

    return nil unless File.exist?(File.join(rust_dir, "Cargo.toml"))
    return nil unless system("cargo --version > /dev/null 2>&1")

    platform = Gem::Platform.local
    platform_key = "#{platform.cpu}-#{platform.os}"
    exe_directory = File.join(File.expand_path(File.join("..", "exe", platform_key), __dir__))
    exe_file = File.join(exe_directory, EXECUTABLE_NAME)

    return exe_file if File.executable?(exe_file)

    warn "yerba: No precompiled binary found. Compiling from source..."

    FileUtils.mkdir_p(exe_directory)

    root_dir = File.expand_path(File.join("..", ".."), __dir__)
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

  def self.parse(content)
    Document.parse(content)
  end

  def self.parse_file(path)
    Document.new(path)
  end

  def self.document(path)
    Document.new(path)
  end

  def self.files(glob)
    Collection.new(glob)
  end
end
