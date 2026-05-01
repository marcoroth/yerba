# frozen_string_literal: true

require "fileutils"

rust_dir = File.expand_path("../../rust", __dir__)
root_dir = File.expand_path("../..", __dir__)

platform = Gem::Platform.local
platform_key = "#{platform.cpu}-#{platform.os}"
exe_directory = File.join(root_dir, "exe", platform_key)
exe_file = File.join(exe_directory, "yerba")

if File.executable?(exe_file)
  puts "yerba: Precompiled binary found at #{exe_file}"
else
  puts "yerba: No precompiled binary found. Compiling from Rust source..."

  unless system("cargo --version > /dev/null 2>&1")
    abort <<~MESSAGE

      ERROR: Rust toolchain not found.

      yerba requires a precompiled binary or the Rust toolchain to compile from source.

      Install Rust: https://rustup.rs

    MESSAGE
  end

  unless system("cd #{rust_dir} && cargo build --release")
    abort "ERROR: Failed to compile yerba from Rust source."
  end

  source_binary = File.join(rust_dir, "target", "release", "yerba")

  unless File.exist?(source_binary)
    abort "ERROR: Compilation succeeded but binary not found at #{source_binary}"
  end

  FileUtils.mkdir_p(exe_directory)
  FileUtils.cp(source_binary, exe_file)
  FileUtils.chmod(0o755, exe_file)

  puts "yerba: Successfully compiled binary to #{exe_file}"
end

File.write("Makefile", <<~MAKEFILE)
  all:
  \t@echo "yerba: nothing to build"
  install:
  \t@echo "yerba: nothing to install"
  clean:
  \t@echo "yerba: nothing to clean"
MAKEFILE
