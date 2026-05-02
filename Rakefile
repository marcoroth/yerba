# frozen_string_literal: true

require "bundler/gem_tasks"
require "rake/testtask"

begin
  require "rake/extensiontask"

  PLATFORMS = [
    "aarch64-linux-gnu",
    "aarch64-linux-musl",
    "arm64-darwin",
    "x86_64-darwin",
    "x86_64-linux-gnu",
    "x86_64-linux-musl"
  ].freeze

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

    exttask.cross_platform.each do |platform|
      desc "Build all native binary gems in parallel"
      multitask "native" => platform

      desc "Build the native gem for #{platform}"
      task platform => "prepare" do
        RakeCompilerDock.sh(
          "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable && " \
          "export PATH=\"$HOME/.cargo/bin:$PATH\" && " \
          "export RCD_PLATFORM=#{platform} && " \
          "bundle --local && rake native:#{platform} gem RUBY_CC_VERSION='#{ENV.fetch("RUBY_CC_VERSION", nil)}'",
          platform: platform
        )
      end
    end
  end
rescue LoadError
  desc "Compile task not available (rake-compiler not installed)"
  task :compile do
    abort "rake-compiler is required. Run: bundle install"
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
