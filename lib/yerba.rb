# frozen_string_literal: true

require_relative "yerba/version"

module Yerba
  class UnsupportedPlatformError < StandardError; end
  class ExecutableNotFoundError < StandardError; end
  class CompilationError < StandardError; end

  GEM_NAME = "yerba"
  EXECUTABLE_NAME = "yerba"

  NATIVE_PLATFORMS = {
    "arm64-darwin" => "arm64-darwin",
    "x86_64-darwin" => "x86_64-darwin",
    "aarch64-linux" => "aarch64-linux",
    "arm64-linux" => "aarch64-linux",
    "x86_64-linux" => "x86_64-linux",
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

      raise ExecutableNotFoundError,
            "yerba executable not found at #{install_dir_exe} (set by YERBA_INSTALL_DIR)"
    end

    # Try platform-specific precompiled binary
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

    # Try compiling from source if Rust source is bundled
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

    unless system("cd #{rust_dir} && cargo build --release")
      raise CompilationError, "Failed to compile yerba from source. Is Rust installed?"
    end

    source_binary = File.join(rust_dir, "target", "release", EXECUTABLE_NAME)

    unless File.exist?(source_binary)
      raise CompilationError, "Compilation succeeded but binary not found at #{source_binary}"
    end

    FileUtils.cp(source_binary, exe_file)
    FileUtils.chmod(0o755, exe_file)

    exe_file
  end

  private_class_method :compile_from_source
end
