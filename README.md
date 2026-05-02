<p align="center">
  <img src="assets/logo.png" alt="Yerba Logo" width="150px">
</p>

<h2 align="center">Yerba</h2>

<h4 align="center"><u>Y</u>AML <u>E</u>diting and <u>R</u>efactoring with <u>B</u>etter <u>A</u>ccuracy</h4>

<div align="center">A CLI tool for editing YAML while preserving structure, comments, and format.</div><br/>

<p align="center">
  <a href="https://rubygems.org/gems/yerba"><img alt="Gem Version" src="https://img.shields.io/gem/v/yerba"></a>
  <a href="https://crates.io/crates/yerba"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/yerba"></a>
  <a href="https://github.com/marcoroth/yerba/blob/main/LICENSE.txt"><img alt="License" src="https://img.shields.io/github/license/marcoroth/yerba"></a>
  <a href="https://github.com/marcoroth/yerba/issues"><img alt="Issues" src="https://img.shields.io/github/issues/marcoroth/yerba"></a>
</p>

<br/>

### What is Yerba?

**Yerba** is a lossless YAML editing tool that lets you programmatically modify YAML files while preserving their original structure, comments, and formatting.

Yerba was born out of the need to manage, validate, programmatically modify, and enforce lossless and consistent style/formatting for the YAML data files in the [RubyEvents.org](https://github.com/rubyevents/rubyevents) project.

### Command-Line Usage

Install the Yerba gem via RubyGems:

```bash
gem install yerba
```

### Examples

```bash
yerba set config.yml path.to.key value
yerba get config.yml path.to.key
```

### Development

After checking out the repo, run `bundle install` to install dependencies. Then, run `bundle exec rake test` to run the tests.

### Rust

```bash
cd rust
cargo build
cargo test
```

### License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
