<p align="center">
  <img src="assets/logo.png" alt="Yerba Logo" width="150px">
</p>

<h2 align="center">Yerba</h2>

<h4 align="center"><u>Y</u>AML <u>E</u>diting and <u>R</u>efactoring with <u>B</u>etter <u>A</u>ccuracy</h4>

<div align="center">A Rust CLI tool and Ruby library for editing YAML while preserving structure, comments, and format.</div><br/>

<p align="center">
  <a href="https://rubygems.org/gems/yerba"><img alt="Gem Version" src="https://img.shields.io/gem/v/yerba"></a>
  <a href="https://crates.io/crates/yerba"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/yerba"></a>
  <a href="https://github.com/marcoroth/yerba/blob/main/LICENSE.txt"><img alt="License" src="https://img.shields.io/github/license/marcoroth/yerba"></a>
  <a href="https://github.com/marcoroth/yerba/issues"><img alt="Issues" src="https://img.shields.io/github/issues/marcoroth/yerba"></a>
</p>

<br/>

## What is Yerba?

Yerba is a lossless YAML editing tool. It lets you programmatically read, modify, and enforce formatting in YAML files while preserving their original structure, including comments, blank lines, quote styles, and key ordering.

Most YAML libraries parse a file into a data structure and serialize it back, discarding all formatting in the process. Yerba operates on the concrete syntax tree (CST), so your edits are surgical: only the targeted values change, and everything else stays exactly as it was.

Yerba is available as:

- A **standalone CLI binary** with zero runtime dependencies
- A **Rust crate** for embedding in Rust applications
- A **Ruby gem** for programmatic YAML editing from Ruby

Yerba was born out of the need to manage, validate, and enforce consistent formatting for hundreds of YAML data files in the [RubyEvents.org](https://github.com/rubyevents/rubyevents) project.

## Installation

### CLI (standalone)

The `yerba` CLI is a standalone Rust binary with no Ruby dependency. Install it via Cargo:

```bash
cargo install yerba
```

### Rust Crate

Use `yerba` as a library in your Rust project:

```toml
[dependencies]
yerba = "0.3"
```

```rust
let mut document = yerba::parse_file("config.yml")?;
document.set("database.host", "0.0.0.0")?;
document.save()?;
```

### Ruby Gem

The Ruby gem bundles both the CLI binary and a native extension for programmatic access from Ruby:

```bash
gem install yerba
```

Or add it to your `Gemfile`:

```ruby
gem "yerba"
```

The gem ships with precompiled binaries for macOS and Linux.

If no precompiled binary is available for your platform, it will compile from source automatically, which requires a [Rust toolchain](https://rustup.rs).

## CLI Usage

The `yerba` CLI follows a consistent pattern:

```
yerba <command> <file> <selector> [value] [options]
```

Selectors use dot-notation for nested keys, brackets for array access, and support glob patterns for operating on multiple files at once.

### Selectors

Selectors let you address any node in a YAML document:

| Pattern | Meaning | Example |
|---------|---------|---------|
| `key` | A single key | `"database.host"` |
| `key.nested` | Nested key path | `"database.settings.pool"` |
| `[]` | All items in array | `"[].title"` |
| `[N]` | Item at index | `"[0].title"` |
| `[].key[].nested` | Nested array access | `"[].speakers[].name"` |

### Conditions

Conditions filter which items a command operates on:

| Syntax | Meaning | Example |
|--------|---------|---------|
| `.key == value` | Equality | `".kind == keynote"` |
| `.key != value` | Inequality | `".status != draft"` |
| `.key contains val` | Substring or member | `".title contains Ruby"` |
| `.key not_contains val` | Negated contains | `".title not_contains test"` |

---

### `get`

Retrieve values from YAML files. Supports single values, array traversal, glob patterns across multiple files, conditions for filtering, and field selection.

```bash
yerba get config.yml "database.host"
yerba get videos.yml "[].title"
yerba get videos.yml "[0].title"
```

Use `--select` to pick specific fields from each item, and `--condition` to filter which items are returned:

```bash
yerba get videos.yml "[]" --select ".title,.speakers"
yerba get videos.yml "[]" --condition ".kind == keynote"
yerba get videos.yml "[]" --select ".title" --condition ".kind == keynote"
```

Glob patterns let you query across many files at once:

```bash
yerba get "data/**/videos.yml" "[].speakers[].name"
yerba get "data/**/videos.yml" "[]" --condition ".kind == keynote" --select ".id,.title"
```

Use `--raw` to output plain values (one per line) instead of JSON:

```bash
yerba get videos.yml "[]" --condition ".speakers contains Matz" --raw
```

### `set`

Update an existing value at a path. The original quote style is preserved automatically, if a value was double-quoted before, it stays double-quoted after the edit.

```bash
yerba set config.yml "database.host" "0.0.0.0"
yerba set videos.yml "[0].title" "New Title"
```

Use `--if-exists` to only set the value when the path already exists, or `--if-missing` to only set it when the path does not exist:

```bash
yerba set config.yml "database.host" "0.0.0.0" --if-exists
yerba set "data/**/event.yml" "website" "" --if-exists
```

Use `--condition` to only apply the change when a sibling field matches:

```bash
yerba set config.yml "database.host" "0.0.0.0" --condition ".port == 5432"
```

Use `--all` to update all nodes matching a wildcard selector:

```bash
yerba set videos.yml "[].description" "" --all
```

### `insert`

Insert a new key into a map or a new item into a sequence. By default, new items are appended at the end.

```bash
yerba insert config.yml "database.ssl" true
yerba insert config.yml "tags" "yaml"
```

Control placement with `--before`, `--after`, or `--at`:

```bash
yerba insert config.yml "database.ssl" true --after "host"
yerba insert config.yml "database.ssl" true --before "port"
yerba insert config.yml "tags" "yaml" --at 0
yerba insert config.yml "tags" "yaml" --after "ruby"
```

For sequences of maps, use conditions to position relative to other items:

```bash
yerba insert speakers.yml "" "name: Bob" --after ".name == Alice"
yerba insert videos.yml "[0].speakers" "Diana" --before ".name == Charlie"
```

Use `--from` to read the value from another file (or stdin with `-`):

```bash
yerba insert videos.yml "" --from "new_talk.yml" --after ".id == first-talk"
```

### `delete`

Remove a key and its value from a map:

```bash
yerba delete config.yml "database.pool"
yerba delete videos.yml "[0].description"
```

Use `--dry-run` to preview the result without writing to the file:

```bash
yerba delete config.yml "database.pool" --dry-run
```

### `remove`

Remove a specific item from a sequence by its value:

```bash
yerba remove config.yml "tags" "rust"
yerba remove videos.yml "[0].speakers" "Alice"
```

### `rename`

Rename a key in a map while preserving its value and position:

```bash
yerba rename config.yml "database.host" "database.hostname"
yerba rename config.yml "database.host" "hostname"
```

### `move`

Move a sequence item to a new position. You can reference items by value, index, or condition:

```bash
yerba move config.yml "tags" "rust" --before "ruby"
yerba move config.yml "tags" "rust" --after "yaml"
yerba move config.yml "tags" 2 --to 0
yerba move videos.yml "" ".id == talk-2" --after ".id == talk-1"
```

### `move-key`

Move a key to a new position within a map:

```bash
yerba move-key config.yml "database.name" --to 0
yerba move-key config.yml "database.pool" --before "database.host"
yerba move-key config.yml "database.pool" --after "database.name"
```

### `sort`

Sort items in a sequence. For simple scalar sequences, no options are needed. For sequences of maps, use `--by` to specify the sort field. Use `--order desc` for descending. Repeat `--by` and `--order` for tie-breakers:

```bash
yerba sort config.yml "tags"
yerba sort videos.yml --by ".title"
yerba sort videos.yml --by ".date" --order desc --by ".title"
yerba sort videos.yml "[].speakers" --by ".name"
```

Use `--order` with a comma-separated list to specify an explicit custom order. All items must be listed:

```bash
yerba sort videos.yml "[]" --by ".id" --order "talk-c,talk-a,talk-b"
yerba sort speakers.yml "[]" --by ".name" --order "Charlie,Alice,Bob"
yerba sort config.yml "tags" --by "." --order "yaml,ruby,rust"
```

This is useful for reordering items in a specific sequence (e.g., conference schedule order, priority lists) or when an LLM agent needs to rearrange items programmatically.

### `sort-keys`

Reorder the keys in a map to match a predefined order. If any key in the document is not present in the order list, the command aborts with an error, this ensures you account for every field:

```bash
yerba sort-keys config.yml "database" "host,port,name,pool"
yerba sort-keys "data/**/event.yml" "" "id,title,kind,location"
yerba sort-keys "data/**/videos.yml" "[]" "id,title,speakers"
```

### `quote-style`

Enforce a consistent quote style across keys and/or values. Available styles are `plain`, `single`, and `double`:

```bash
yerba quote-style config.yml --values double
yerba quote-style config.yml --keys plain
yerba quote-style config.yml --keys plain --values double
```

Scope the operation to a specific selector:

```bash
yerba quote-style config.yml "[].speakers" --values plain
yerba quote-style "data/**/*.yml" --keys plain --values double
```

### `blank-lines`

Enforce a consistent number of blank lines between sequence entries:

```bash
yerba blank-lines videos.yml 1
yerba blank-lines videos.yml "[]" 1
yerba blank-lines config.yml "tags" 0
```

### `selectors`

Show all valid selectors for a YAML file. Useful for discovering the structure of a file and knowing which selectors you can use with other commands:

```bash
yerba selectors config.yml
```

Output:
```
database
database.host
database.port
tags
tags[]
```

For sequences of objects:

```bash
yerba selectors videos.yml
```

Output:
```
[]
[].id
[].title
[].speakers
[].speakers[]
[].speakers[].name
[].speakers[].slug
[].video_id
[].video_provider
```

Pass a selector to scope the output to a specific subtree:

```bash
yerba selectors config.yml "database"
yerba selectors videos.yml "[]"
yerba selectors videos.yml "[].speakers"
```

Works with glob patterns to show the union of selectors across multiple files:

```bash
yerba selectors "data/**/videos.yml"
yerba selectors "data/**/videos.yml" "[]"
```

## `Yerbafile`

A `Yerbafile` is a YAML configuration file that defines formatting and editing rules as pipelines of operations that are applied to your files across your project.

Use `yerba init` to create one, then `yerba apply` to apply all rules, or `yerba check` to verify compliance (exits with code `1` if files would change):

```bash
yerba init
yerba apply
yerba check
```

Each rule specifies a file glob and a list of steps to run in order:

```yaml
rules:
  - files: "config/**/*.yml"
    pipeline:
      - quote_style:
          key_style: plain
          value_style: double

      - sort_keys:
          path: ""
          order:
            - id
            - title
            - description

      - blank_lines:
          count: 1

  - files: "data/speakers.yml"
    pipeline:
      - quote_style:
          key_style: plain
          value_style: double

      - sort_keys:
          path: ""
          order:
            - name
            - slug
            - github
            - twitter
            - website

      - sort:
          path: ""
          by: name
```

Available pipeline steps:

- `quote_style` Enforce quote style on keys and/or values, optionally scoped by path
- `sort_keys` Reorder keys to match a predefined list
- `sort` Sort sequence items by field(s)
- `blank_lines` Enforce blank lines between sequence entries
- `set` Set a value (supports conditions)
- `insert` Insert a new key or sequence item
- `delete` Remove a key (supports conditions)
- `rename` Rename a key
- `remove` Remove an item from a sequence
- `get` Read a value and store it as a variable for subsequent steps

This makes it easy to enforce project-wide YAML conventions in CI:

```bash
yerba check
```

## Ruby API

Yerba includes a native C extension (backed by the same Rust core) that provides a full Ruby API for YAML editing.

### Parsing

Create a document from a file path or from a string:

```ruby
require "yerba"

document = Yerba.parse_file("config.yml")
document = Yerba.parse("database:\n  host: localhost\n  port: 5432\n")
```

### Reading Values

Use `get` to retrieve the raw value at a path. Values are returned with their YAML types, strings, integers, booleans, and nil are all mapped to their Ruby equivalents:

```ruby
document.get("database.host")   # => "localhost"
document.get("database.port")   # => 5432
document.get("database.ssl")    # => false
```

### Structured Navigation

Use bracket notation to get typed wrapper objects (`Scalar`, `Map`, or `Sequence`) representing nodes in the document. These are live references, mutations flow back to the document.

You can use a full dot-path in a single bracket call, or chain brackets to navigate one level at a time:

```ruby
document["database.host"].value          # => "localhost"
document["database"]["host"].value       # => "localhost"
document["database"]["port"].value       # => 5432
```

The returned object type depends on what's at the path:

```ruby
document["database"]       # => Yerba::Map
document["database.host"]  # => Yerba::Scalar
document["tags"]           # => Yerba::Sequence
```

Scalars expose their value and quote style:

```ruby
scalar = document["database.host"]
scalar.value         # => "localhost"
scalar.quote_style   # => :double
```

### Mutations

Modify values in place. The original formatting is preserved:

```ruby
document["database"]["host"].value = "0.0.0.0"
document.set("database.port", 3306)
```

Set all matching nodes at once with `all: true`:

```ruby
document.set("[].description", "", all: true)
```

Insert new keys with positional control:

```ruby
document["database"].insert("ssl", true, after: "host")
```

Work with sequences using familiar Ruby patterns:

```ruby
tags = document["tags"]
tags << "yaml"
tags << { name: "Rust", version: "1.80" }
tags.remove("obsolete")
```

### Sorting

Sort sequences in place. Works on both the document and sequence level:

```ruby
document.sort(by: :name)
document.sort(by: :name, order: :desc)
document.sort(by: :name, order: ["Charlie", "Bob", "Alice"])
document.sort("tags")
document.sort("tags", order: :desc)
document.sort("tags", order: ["rust", "ruby", "go"])
```

The `by:` option accepts symbols, strings, or dot-prefixed strings (`:name`, `"name"`, `".name"`).

### Querying

Find and filter items in sequences with `find_by`, `where`, and `pluck`:

```ruby
document.find_by(name: "Alice")
document.where(role: "admin")
document.pluck(:name)

document.find_by(speakers: { name: "Alice" })
document.where(tags: ["ruby"])
document.find_by("database.host": "localhost")
```

These methods work on `Document` (delegates to root), `Sequence`, and `Collection` (searches across files):

```ruby
collection = Yerba.files("data/**/*.yml")
collection.find_by(name: "Alice")
collection.where(kind: "talk")
collection.pluck(:name)
```

### Quote Style Control

Read and set the quote style on individual scalars:

```ruby
scalar = document["database.host"]
scalar.quote_style # => :double
scalar.quote_style = :single
```

### Wildcard Access

Use `at_path` to access all items matching a wildcard selector:

```ruby
titles = document.at_path("[].title")
titles.each { |scalar| puts scalar.value }
```

### Collections

Operate on multiple files matching a glob pattern:

```ruby
collection = Yerba.files("data/**/videos.yml")

collection.each do |document|
  puts document.get("[0].title")
end

collection.find_by(name: "Alice")
collection.where(kind: "talk")
collection.pluck(:name)

collection.apply! do |document|
  document.set("status", "published")
end
```

### Saving

Write changes back to the original file:

```ruby
document.save!
```

Or render the document as a string without writing to disk:

```ruby
document.to_s
```

## Development

After checking out the repo, run `bundle install` to install Ruby dependencies, then `bundle exec rake test` to run the test suite.

### Building from source

The Rust core is in the `rust/` directory:

```bash
cd rust
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
cd rust
cargo build --release
./target/release/yerba --help
```

## License

The gem is available as open source under the terms of the [MIT License](https://github.com/marcoroth/yerba/blob/main/LICENSE.txt).
