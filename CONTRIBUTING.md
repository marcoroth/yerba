# Contributing

## Development

After checking out the repo, run `bundle install` to install Ruby dependencies, then `bundle exec rake test` to run the test suite.

### Building from source

The Rust core is in the `rust/` directory, with a workspace `Cargo.toml` at the root so all cargo commands work from the project root:

```bash
cargo build
cargo test
```

The C extension (for the Ruby API) is compiled via `ext/yerba/extconf.rb` which invokes `cargo build` and links against the resulting static library. Running `bundle exec rake compile` will build both the Rust library and the C extension.

### Running the CLI locally

```bash
cargo run -- get config.yml "database.host"
```

Or build a release binary:

```bash
cargo build --release
./target/release/yerba --help
```