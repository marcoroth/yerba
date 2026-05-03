# frozen_string_literal: true

begin
  require_relative "lib/yerba/version"
rescue LoadError
  puts "WARNING: Could not load Yerba::VERSION"
end

Gem::Specification.new do |spec|
  spec.name = "yerba"
  spec.version = defined?(Yerba::VERSION) ? Yerba::VERSION : "0.0.0"
  spec.authors = ["Marco Roth"]
  spec.email = ["marco.roth@intergga.ch"]

  spec.summary = "YAML Editing and Refactoring with Better Accuracy"
  spec.description = "A CLI tool for editing YAML while preserving structure, comments, and format."
  spec.homepage = "https://github.com/marcoroth/yerba"
  spec.license = "MIT"

  spec.required_ruby_version = ">= 3.2.0"
  spec.require_paths = ["lib"]

  spec.files = Dir[
    "yerba.gemspec",
    "LICENSE.txt",
    "README.md",
    "lib/**/*.rb",
    "sig/**/*.rbs",
    "exe/yerba",
    "exe/*/yerba",
    "ext/yerba/extconf.rb",
    "ext/yerba/yerba.c",
    "ext/yerba/include/**/*.h",
    "rust/build.rs",
    "rust/cbindgen.toml",
    "rust/Cargo.toml",
    "rust/Cargo.lock",
    "rust/src/**/*.rs",
    "rust/rustfmt.toml"
  ]

  spec.bindir = "exe"
  spec.executables = ["yerba"]
  spec.extensions = ["ext/yerba/extconf.rb"]

  spec.metadata["allowed_push_host"] = "https://rubygems.org"
  spec.metadata["rubygems_mfa_required"] = "true"

  spec.metadata["homepage_uri"] = "https://github.com/marcoroth/yerba"
  spec.metadata["changelog_uri"] = "https://github.com/marcoroth/yerba/releases"
  spec.metadata["source_code_uri"] = "https://github.com/marcoroth/yerba"
  spec.metadata["bug_tracker_uri"] = "https://github.com/marcoroth/yerba/issues"
end
