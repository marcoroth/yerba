# frozen_string_literal: true

require "rubygems/package"

NATIVE_PLATFORMS = {
  "arm64-darwin" => { target: "aarch64-apple-darwin" },
  "x86_64-darwin" => { target: "x86_64-apple-darwin" },
  "aarch64-linux" => { target: "aarch64-unknown-linux-gnu", linker: "aarch64-linux-gnu-gcc" },
  "x86_64-linux" => { target: "x86_64-unknown-linux-gnu" },
}.freeze

desc "Compile Rust binary for the current platform"
task :compile do
  sh "cd rust && cargo build --release"

  platform = Gem::Platform.local
  platform_key = "#{platform.cpu}-#{platform.os}"
  exe_directory = File.join("exe", platform_key)

  mkdir_p exe_directory

  binary_name = Gem.win_platform? ? "yerba.exe" : "yerba"
  cp "rust/target/release/#{binary_name}", File.join(exe_directory, binary_name)
  chmod 0o755, File.join(exe_directory, binary_name)
end

desc "Cross-compile Rust binary for a specific target"
task :cross_compile, [:platform] do |_task, args|
  platform = args[:platform]
  config = NATIVE_PLATFORMS[platform]

  unless config
    abort "Unknown platform: #{platform}. Known platforms: #{NATIVE_PLATFORMS.keys.join(", ")}"
  end

  rust_target = config[:target]

  sh "rustup target add #{rust_target}"

  env_vars = {}

  if config[:linker]
    linker_env = "CARGO_TARGET_#{rust_target.upcase.tr("-", "_")}_LINKER"
    env_vars[linker_env] = config[:linker]
  end

  env_string = env_vars.map { |key, value| "#{key}=#{value}" }.join(" ")
  sh "cd rust && #{env_string} cargo build --release --target #{rust_target}"

  exe_directory = File.join("exe", platform)
  mkdir_p exe_directory

  binary_name = "yerba"
  cp "rust/target/#{rust_target}/release/#{binary_name}", File.join(exe_directory, binary_name)
  chmod 0o755, File.join(exe_directory, binary_name)
end

desc "Build platform-specific gem for a given platform"
task :gem, [:platform] do |_task, args|
  platform = args[:platform]

  exe_directory = File.join("exe", platform)

  unless File.exist?(File.join(exe_directory, "yerba"))
    abort "Binary not found at #{exe_directory}/yerba. Run rake cross_compile[#{platform}] first."
  end

  gemspec = Gem::Specification.load("yerba.gemspec")
  gemspec.platform = Gem::Platform.new(platform)
  gemspec.files += Dir["#{exe_directory}/**/*"]

  package = Gem::Package.build(gemspec)
  mkdir_p "pkg"
  mv package, "pkg/"
end

desc "Build the pure Ruby (fallback) gem"
task "gem:ruby" do
  sh "gem build yerba.gemspec"

  mkdir_p "pkg"
  mv Dir["yerba-*.gem"].first, "pkg/"
end

desc "Build all platform-specific gems"
task "gem:all" do
  NATIVE_PLATFORMS.each_key do |platform|
    Rake::Task[:gem].reenable
    Rake::Task[:gem].invoke(platform)
  end

  Rake::Task["gem:ruby"].invoke
end

desc "Cross-compile and package all platform gems"
task :package do
  NATIVE_PLATFORMS.each_key do |platform|
    Rake::Task[:cross_compile].reenable
    Rake::Task[:cross_compile].invoke(platform)
  end

  Rake::Task["gem:all"].invoke
end
