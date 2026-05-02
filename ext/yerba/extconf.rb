# frozen_string_literal: true

require "mkmf"
require "fileutils"

rust_dir = File.expand_path("../../rust", __dir__)
root_dir = File.expand_path("../..", __dir__)
lib_dir = File.join(rust_dir, "target", "release")

unless system("cargo --version > /dev/null 2>&1")
  abort <<~MESSAGE

    ERROR: Rust toolchain not found.

    yerba requires the Rust toolchain to compile from source.

    Install Rust: https://rustup.rs

  MESSAGE
end

puts "yerba: Compiling Rust library and binary..."

unless system("cd #{rust_dir} && cargo build --release")
  abort "ERROR: Failed to compile yerba from Rust source."
end

platform = Gem::Platform.local
platform_key = "#{platform.cpu}-#{platform.os}"
exe_directory = File.join(root_dir, "exe", platform_key)
exe_file = File.join(exe_directory, "yerba")
source_binary = File.join(rust_dir, "target", "release", "yerba")

if File.exist?(source_binary)
  FileUtils.mkdir_p(exe_directory)
  FileUtils.cp(source_binary, exe_file)
  FileUtils.chmod(0o755, exe_file)

  puts "yerba: CLI binary installed to #{exe_file}"
end

lib_name = case RbConfig::CONFIG["host_os"]
           when /darwin/
             "libyerba.dylib"
           when /mingw|mswin/
             "yerba.dll"
           else
             "libyerba.so"
           end

lib_path = File.join(lib_dir, lib_name)

unless File.exist?(lib_path)
  abort "ERROR: Shared library not found at #{lib_path}"
end

puts "yerba: Shared library found at #{lib_path}"

$LDFLAGS << " -L#{lib_dir} -lyerba"
$CFLAGS << " -I#{File.join(__dir__, "include")}"

case RbConfig::CONFIG["host_os"]
when /darwin/, /linux/
  $LDFLAGS << " -Wl,-rpath,#{lib_dir}"
end

create_makefile("yerba/yerba")
