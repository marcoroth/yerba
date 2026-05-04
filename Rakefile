# frozen_string_literal: true

require "bundler/gem_tasks"
require "rake/testtask"

begin
  require "rake/extensiontask"
  require "rb_sys"

  PLATFORMS = [
    "aarch64-linux-gnu",
    "aarch64-linux-musl",
    "arm-linux-gnu",
    "arm64-darwin",
    "x86_64-darwin",
    "x86_64-linux-gnu",
    "x86_64-linux-musl"
  ].freeze

  RB_SYS_PLATFORM_MAP = {
    "aarch64-linux-gnu" => "aarch64-linux",
    "aarch64-linux-musl" => "aarch64-linux-musl",
    "arm-linux-gnu" => "arm-linux",
    "arm64-darwin" => "arm64-darwin",
    "x86_64-darwin" => "x86_64-darwin",
    "x86_64-linux-gnu" => "x86_64-linux",
    "x86_64-linux-musl" => "x86_64-linux-musl",
  }.freeze

  exttask = Rake::ExtensionTask.new do |ext|
    ext.name = "yerba"
    ext.source_pattern = "*.{c,h}"
    ext.ext_dir = "ext/yerba"
    ext.lib_dir = "lib/yerba"
    ext.gem_spec = Gem::Specification.load("yerba.gemspec")
    ext.cross_compile = true
    ext.cross_platform = PLATFORMS
  end

  namespace "gem" do
    task "prepare" do
      require "rake_compiler_dock"

      sh "bundle config set cache_all true"

      gemspec_path = File.expand_path("./yerba.gemspec", __dir__)
      spec = eval(File.read(gemspec_path), binding, gemspec_path) # rubocop:disable Security/Eval

      RakeCompilerDock.set_ruby_cc_version(spec.required_ruby_version.as_list)
    rescue LoadError
      abort "rake_compiler_dock is required for this task"
    end

    PLATFORMS.each do |platform|
      desc "Build all native binary gems in parallel"
      multitask "native" => platform

      desc "Build the native gem for #{platform}"
      task platform => "prepare" do
        rb_sys_platform = RB_SYS_PLATFORM_MAP.fetch(platform)

        rust_target = {
          "aarch64-linux-gnu" => "aarch64-unknown-linux-gnu",
          "aarch64-linux-musl" => "aarch64-unknown-linux-musl",
          "arm-linux-gnu" => "armv7-unknown-linux-gnueabihf",
          "arm-linux-musl" => "armv7-unknown-linux-musleabihf",
          "arm64-darwin" => "aarch64-apple-darwin",
          "x86_64-darwin" => "x86_64-apple-darwin",
          "x86_64-linux-gnu" => "x86_64-unknown-linux-gnu",
          "x86_64-linux-musl" => "x86_64-unknown-linux-musl",
        }.fetch(platform)

        linker_env = case rust_target
                     when /aarch64.*linux.*gnu/ then "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc"
                     when /aarch64.*linux.*musl/ then "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc"
                     when /armv7.*gnueabihf/ then "CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc"
                     else ""
                     end

        RakeCompilerDock.sh(
          "rustup target add #{rust_target} 2>/dev/null; " \
          "export #{linker_env}; " \
          "cargo build --release --target #{rust_target} && " \
          "mkdir -p exe/#{platform} && cp target/#{rust_target}/release/yerba exe/#{platform}/yerba && " \
          "export RCD_PLATFORM=#{platform} && " \
          "bundle --local && rake native:#{platform} gem RUBY_CC_VERSION='#{ENV.fetch("RUBY_CC_VERSION", nil)}'",
          platform: platform,
          image: "rbsys/#{rb_sys_platform}:#{RbSys::VERSION}"
        )
      end
    end
  end
rescue LoadError => e
  warn "WARNING: Failed to load extension tasks: #{e.message}"

  desc "Compile task not available (rake-compiler not installed)"
  task :compile do
    abort "rake-compiler is required: #{e.message}\n\nRun: bundle install"
  end
end

Rake::TestTask.new(:test) do |t|
  t.libs << "test"
  t.libs << "lib"
  t.libs << "ext"
  t.test_files = FileList["test/**/*_test.rb"]
end

task test: :compile
task default: [:test]

Dir["rakelib/**/*.rake"].each { |f| load f }
