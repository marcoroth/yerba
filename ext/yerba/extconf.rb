# frozen_string_literal: true

require "mkmf"
require "fileutils"

rust_dir = File.expand_path("../../rust", __dir__)
root_dir = File.expand_path("../..", __dir__)

unless system("cargo --version > /dev/null 2>&1")
  abort <<~MESSAGE

    ERROR: Rust toolchain not found.

    yerba requires the Rust toolchain to compile from source.

    Install Rust: https://rustup.rs

  MESSAGE
end

RUST_TARGETS = {
  "aarch64-linux-gnu" => "aarch64-unknown-linux-gnu",
  "aarch64-linux-musl" => "aarch64-unknown-linux-musl",
  "arm64-darwin" => "aarch64-apple-darwin",
  "x86_64-darwin" => "x86_64-apple-darwin",
  "x86_64-linux-gnu" => "x86_64-unknown-linux-gnu",
  "x86_64-linux-musl" => "x86_64-unknown-linux-musl",
}.freeze

cross_compiling = ENV.key?("RUBY_CC_VERSION")
target_platform = ENV.fetch("CARGO_BUILD_TARGET", nil)

if cross_compiling && target_platform.nil?
  ruby_platform = RbConfig::CONFIG["arch"]
  target_platform = RUST_TARGETS.values.find { |t| ruby_platform.include?(t.split("-").first) }

  target_platform ||= RUST_TARGETS[ENV.fetch("RCD_PLATFORM", "")]
end

if target_platform
  puts "yerba: Cross-compiling Rust for target: #{target_platform}"
  system("rustup target add #{target_platform}") || warn("yerba: Failed to add Rust target #{target_platform}")

  cargo_args = "--release --target #{target_platform}"
  lib_dir = File.join(rust_dir, "target", target_platform, "release")
else
  puts "yerba: Compiling Rust library for native platform..."
  cargo_args = "--release"
  lib_dir = File.join(rust_dir, "target", "release")
end

unless system("cd #{rust_dir} && cargo build #{cargo_args}")
  abort "ERROR: Failed to compile yerba from Rust source."
end

platform = Gem::Platform.local
platform_key = "#{platform.cpu}-#{platform.os}"
exe_directory = File.join(root_dir, "exe", platform_key)
exe_file = File.join(exe_directory, "yerba")
source_binary = File.join(lib_dir, "..", "..", "release", "yerba")
source_binary = File.join(lib_dir, "yerba") unless File.exist?(source_binary)

if File.exist?(source_binary)
  FileUtils.mkdir_p(exe_directory)
  FileUtils.cp(source_binary, exe_file)
  FileUtils.chmod(0o755, exe_file)

  puts "yerba: CLI binary installed to #{exe_file}"
end

lib_name = case RbConfig::CONFIG["host_os"]
           when /darwin/ then "libyerba.dylib"
           when /mingw|mswin/ then "yerba.dll"
           else "libyerba.so"
           end

if target_platform
  lib_name = case target_platform
             when /darwin/ then "libyerba.dylib"
             when /windows|mingw/ then "yerba.dll"
             else "libyerba.so"
             end
end

lib_path = File.join(lib_dir, lib_name)

unless File.exist?(lib_path)
  abort "ERROR: Shared library not found at #{lib_path}"
end

puts "yerba: Shared library found at #{lib_path}"

$LDFLAGS << " -L#{lib_dir} -lyerba"
$CFLAGS << " -I#{File.join(__dir__, "include")}"

if RbConfig::CONFIG["host_os"].match?(/darwin|linux/)
  $LDFLAGS << " -Wl,-rpath,#{lib_dir}"
end

create_makefile("yerba/yerba")
