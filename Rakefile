# frozen_string_literal: true

require "bundler/gem_tasks"
require "rake/testtask"
require "rake/extensiontask"

CROSS_PLATFORMS = [
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
  ext.cross_platform = CROSS_PLATFORMS
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

namespace "gem" do
  task "native" do
    require "rake_compiler_dock"

    CROSS_PLATFORMS.each do |platform|
      RakeCompilerDock.sh "bundle --local && rake native:#{platform} gem", platform: platform
    end
  end

  task "prepare" do
    sh "bundle config set cache_all true"
  end

  exttask.cross_platform.each do |platform|
    task platform => "prepare" do
      require "rake_compiler_dock"
      RakeCompilerDock.sh(
        "bundle --local && rake native:#{platform} gem",
        platform: platform
      )
    end
  end
end
