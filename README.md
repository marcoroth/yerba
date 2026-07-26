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
yerba = "0.8"
```

```rust
let mut document = yerba::parse_file("config.yml")?;
document.set("database.host", "0.0.0.0")?;

document.save()?;                  // saves to original path
document.save_to("output.yml")?;   // saves to new path
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

| Pattern           | Meaning             | Example                    |
|-------------------|---------------------|----------------------------|
| `key`             | A single key        | `"database.host"`          |
| `key.nested`      | Nested key path     | `"database.settings.pool"` |
| `[]`              | All items in array  | `"[].title"`               |
| `[N]`             | Item at index       | `"[0].title"`              |
| `*`               | All values in a map | `"database.*"`             |
| `[].key[].nested` | Nested array access | `"[].speakers[].name"`     |

`[]` and `*` are the same idea applied to the two collection types: `[]` matches every item of a sequence, `*` matches every value of a map. Both can be combined with the other patterns, as in `"[].*"` or `"*.city"`.

### Conditions

Conditions filter which items a command operates on:

| Syntax                  | Meaning             | Example                      |
|-------------------------|---------------------|------------------------------|
| `.key == value`         | Equality            | `".kind == keynote"`         |
| `.key != value`         | Inequality          | `".status != draft"`         |
| `.key contains val`     | Substring or member | `".title contains Ruby"`     |
| `.key not_contains val` | Negated contains    | `".title not_contains test"` |

The selector is relative to each item and starts with `.`. A condition that is malformed, or that uses an absolute selector where each item is tested, is rejected rather than matching nothing, so a typo is an error instead of a command that quietly does nothing.

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

Enforce a consistent quote style across keys and/or values:

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

Use block scalar styles to enforce multiline formatting on specific fields:

```bash
yerba quote-style videos.yml "[].description" --values literal
```

**Key styles** (`--keys`):

| Style    | Symbol | Example         |
|----------|--------|-----------------|
| `plain`  | —      | `host: value`   |
| `single` | `'`    | `'host': value` |
| `double` | `"`    | `"host": value` |

**Value styles** (`--values`):

| Style          | Symbol | Example                  | Behavior                             |                            |
|----------------|--------|--------------------------|--------------------------------------|----------------------------|
| `plain`        | —      | `host: localhost`        | Unquoted                             |                            |
| `single`       | `'`    | `host: 'localhost'`      | Single-quoted                        |                            |
| `double`       | `"`    | `host: "localhost"`      | Double-quoted, supports `\n` escapes |                            |
| `literal`      | `\     | -`                       | Preserves newlines                   | Strip trailing newline     |
| `literal-clip` | `\     | `                        | Preserves newlines                   | Keep one trailing newline  |
| `literal-keep` | `\     | +`                       | Preserves newlines                   | Keep all trailing newlines |
| `folded`       | `>-`   | Folds newlines to spaces | Strip trailing newline               |                            |
| `folded-clip`  | `>`    | Folds newlines to spaces | Keep one trailing newline            |                            |
| `folded-keep`  | `>+`   | Folds newlines to spaces | Keep all trailing newlines           |                            |

Block scalars are only converted when scoped to a specific selector. An unscoped `--values double` will not touch existing block scalars.

### `blank-lines`

Enforce a consistent number of blank lines between sequence entries:

```bash
yerba blank-lines videos.yml 1
yerba blank-lines videos.yml "[]" 1
yerba blank-lines config.yml "tags" 0
```

### `directives`

Add or remove the document start marker (`---`):

```bash
yerba directives config.yml --ensure
yerba directives config.yml --remove
yerba directives "data/**/*.yml" --ensure
```

### `unique`

Find or remove duplicate items in a sequence. Use `--by` to specify which field determines uniqueness:

```bash
yerba unique videos.yml --by ".id"
yerba unique speakers.yml --by ".name"
yerba unique config.yml "tags" --by "."
```

By default, duplicates are reported but not removed. Use `--remove` to remove them (keeps the first occurrence):

```bash
yerba unique videos.yml --by ".id" --remove
yerba unique speakers.yml --by ".name" --remove --dry-run
```

### `location`

Show the location (line, column, byte offset) of a selector in a YAML file:

```bash
yerba location config.yml "database.host"
yerba location videos.yml "[0].title"
yerba location videos.yml "[0]"
```

Output:
```json
{
  "selector": "[0].title",
  "file": "videos.yml",
  "start_line": 2,
  "start_column": 9,
  "end_line": 2,
  "end_column": 19,
  "start_offset": 22,
  "end_offset": 32
}
```

### `schema`

Validate YAML files against a JSON schema:

```bash
yerba schema data/speakers.yml --schema lib/schemas/speaker_schema.json
yerba schema "data/**/videos.yml" --schema lib/schemas/video_schema.json
```

Use `--path` to scope validation to a specific selector (e.g. validate each item in an array):

```bash
yerba schema data/speakers.yml --schema speaker_schema.json --selector "[]"
yerba schema data/sponsors.yml --schema tier_schema.json --selector "tiers[]"
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
yerba apply path/to/file.yml

yerba check
yerba check path/to/file.yml
```

A Yerbafile supports a global `pipeline` that runs on all matching files, plus per-rule pipelines for file-specific steps. Global steps run first, then per-rule steps refine or override:

```yaml
files: "data/**/*.yml"

pipeline:
  - directives:
      max: 1
      ensure: true

  - collection_style:
      style: block

  - sequence_indent:
      style: indented

  - quote_style:
      key_style: plain
      value_style: double

rules:
  - files: "data/**/videos.yml"
    pipeline:
      - quote_style:
          path: "[].speakers"
          value_style: plain

      - sort_keys:
          path: "[]"
          order:
            - id
            - title
            - speakers

  - files: "data/speakers.yml"
    pipeline:
      - sort_keys:
          path: "[]"
          order:
            - name
            - slug
            - github

      - sort:
          by: name
```

Available pipeline steps:

- `quote_style` Enforce quote style on keys and/or values, optionally scoped by path
- `collection_style` Enforce flow or block style on collections
- `sequence_indent` Enforce compact or indented sequence style
- `sort_keys` Reorder keys to match a predefined list
- `sort` Sort sequence items by field(s)
- `blank_lines` Enforce blank lines between sequence entries
- `set` Set a value (supports conditions)
- `insert` Insert a new key or sequence item
- `delete` Remove a key (supports conditions)
- `rename` Rename a key
- `remove` Remove an item from a sequence
- `directives` Add or remove the document start marker (`---`), with optional `max` validation
- `unique` Find or remove duplicate items in a sequence
- `schema` Validate against a JSON schema (with optional `path` for scoping)
- `final_newline` Enforce exact number of trailing newlines (`count: 1` by default, strips extras)

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

document = Yerba.parse(<<~YAML)
  database:
    host: localhost
    port: 5432
YAML
```

### Creating Documents

Build documents from Ruby objects:

```ruby
document = Yerba::Document.from({ name: "Alice", tags: ["ruby", "rails"] })
document = Yerba::Document.from([{ id: "talk-1", title: "First Talk" }])
```

Build incrementally using `Document.new` and `root=`:

```ruby
document = Yerba::Document.new
document.root = {}

document["name"] = "Event 123"
document["kind"] = "conference"
document["tags"] = ["ruby", "rails"]
document["config"] = { host: "localhost", port: 5432 }

document.save_to!("event.yml")
```

```yaml
---
name: Event 123
kind: conference
tags:
  - ruby
  - rails
config:
  host: localhost
  port: 5432
```

Or start with `Document.from` and keep building:

```ruby
document = Yerba::Document.from({ name: "Event 123" })

document["tags"] = []
document["tags"] << "ruby"
document["tags"] << "rails"
```

Build a sequence document and append entries:

```ruby
document = Yerba::Document.new
document.root = []

document << { id: "talk-1", title: "First Talk", speakers: ["Alice"] }
document << { id: "talk-2", title: "Second Talk", speakers: ["Bob", "Carol"] }

document.save_to!("videos.yml")
```

`document["key"]` and `document << item` are shortcuts for `document.root["key"]` and `document.root << item`.

### Reading Values

Use bracket notation (`[]`) to navigate the document. Returns typed node objects (`Scalar`, `Map`, or `Sequence`) that are live references — mutations flow back to the document.

All access methods (`[]`, `fetch`, `dig`, `value_at`) accept full selector strings like `"database.host"`, `"[0].title"`, or `"[].speakers[].name"`. In the examples below we prefer the more idiomatic chained bracket style, but the two forms are equivalent:

```ruby
document["database"]["host"].value  # => "localhost"
document["database.host"].value     # => "localhost" (same thing)
```

The returned object type depends on what's at the path:

```ruby
document["database"]          # => Yerba::Map
document["database"]["host"]  # => Yerba::Scalar
document["tags"]              # => Yerba::Sequence
```

Scalars expose their value and quote style:

```ruby
scalar = document["database"]["host"]
scalar.value        # => "localhost"
scalar.quote_style  # => :double
```

Use `fetch` for strict access, it raises `Yerba::SelectorNotFoundError` with "did you mean?" suggestions if the selector doesn't exist:

```ruby
document.fetch("database.host")  # => Yerba::Scalar
document.fetch("databse.host")   # => raises SelectorNotFoundError: ... Did you mean: database.host?
```

Use `dig` to traverse multiple levels, returning `nil` for missing paths:

```ruby
document.dig("database", "host")     # => Yerba::Scalar
document.dig("items", 0, "name")     # => Yerba::Scalar
document.dig("database", "missing")  # => nil
```

Use `value_at` to get the plain Ruby value (String, Integer, Hash, Array, etc.) instead of a node object:

```ruby
document.value_at("database.host")  # => "localhost"
document.value_at("database.port")  # => 5432
document.value_at("database")       # => {"host" => "localhost", "port" => 5432}
document.value_at("[].title")       # => ["First Talk", "Second Talk"]
```

Summary of access methods:

| Method     | Not found                      | Returns                            |
|------------|--------------------------------|------------------------------------|
| `[]`       | `nil`                          | `Scalar` / `Map` / `Sequence` node |
| `fetch`    | raises `SelectorNotFoundError` | `Scalar` / `Map` / `Sequence` node |
| `dig`      | `nil`                          | `Scalar` / `Map` / `Sequence` node |
| `value_at` | `nil`                          | plain Ruby value                   |

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

Setting a scalar over a key whose value is a collection replaces the collection:

```ruby
document.root["venue"] = "none"  # => venue: none
```

Insert new keys with positional control:

```ruby
document["database"].insert("ssl", true, after: "host")
```

Set arrays and hashes as values, they default to block style:

```ruby
document["database"]["tags"] = ["ruby", "rails"]
# tags:
#   - ruby
#   - rails

document["database"]["settings"] = { pool: 5, timeout: 30 }
# settings:
#   pool: 5
#   timeout: 30
```

Use `set` with `style: :flow` for inline formatting:

```ruby
document.root.set("tags", ["ruby", "rails"], style: :flow)
# tags: [ruby, rails]
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

### Schema Validation

Validate documents against JSON schemas from Ruby:

```ruby
schema = {
  type: "object",
  properties: { name: { type: "string" }, slug: { type: "string" } },
  required: ["name", "slug"]
}

document.valid?(schema)                 # => true/false
document.valid?(schema, selector: "[]") # validate each array item

errors = document.validate(schema, selector: "[]")

errors.each do |error|
  puts "#{error["message"]} at #{error["path"]} (line #{error["line"]})"
end
```

Also accepts a JSON string:

```ruby
document.valid?('{"type":"object","required":["name"]}')
```

### Quote Style Control

Read and set the quote style on individual scalars:

```ruby
scalar = document["database"]["host"]
scalar.quote_style # => :double
scalar.quote_style = :single
```

### Collection Style

Read and change how collections are rendered, flow (inline) or block (multi-line):

```ruby
document["tags"].collection_style          # => :block or :flow
document["tags"].collection_style = :flow  # => tags: [ruby, rails]
document["tags"].collection_style = :block # => tags:\n  - ruby\n  - rails
```

Works on both sequences and maps:

```ruby
document["database"].collection_style = :flow  # => database: {host: localhost, port: 5432}
document["database"].collection_style = :block # => database:\n  host: localhost\n  port: 5432
```

Flow collections are read and iterated like block ones, and their scalars can be
changed in place:

```ruby
document = Yerba::Document.parse("tags: [ruby, rails]\n")

document["tags"].map(&:value)   # => ["ruby", "rails"]
document["tags[1]"].value = "crystal"  # => tags: [ruby, crystal]
```

Adding or removing entries is not supported yet, and raises rather than
reformatting the collection. Switch to block style first:

```ruby
document["tags"].collection_style = :block
document.delete("tags[0]")
```

### Sequence Indent

Control whether sequence items are indented under their key or at the same level:

```ruby
document["tags"].sequence_indent             # => :indented or :compact
document["tags"].sequence_indent = :compact  # compact style
document["tags"].sequence_indent = :indented # indented style
```

```yaml
# :compact               # :indented
tags:                    tags:
- ruby                     - ruby
- rails                    - rails
```

### Location

Get the precise location (line, column, byte offset) of any selector in a document:

```ruby
loc = document[0]["title"].location
loc.start_line    # => 2
loc.start_column  # => 9
loc.end_line      # => 2
loc.end_column    # => 19
loc.start_offset  # => 22
loc.end_offset    # => 32
```

You can also get a location by selector string:

```ruby
document.location("[0].title")
```

The above is the same as:

```ruby
document[0]["title"].location
document["[0].title"].location
```

Omit the selector to get the whole document's location:

```ruby
document.location # => #<Yerba::Location start_line=1, ...>
```

Returns `nil` for non-existent selectors. Use `locations` for wildcard selectors that match multiple nodes:

```ruby
locs = document.locations("[].title")
locs.each { |loc| puts "line #{loc.start_line}" }
# line 2
# line 4

document.locations("[]")
document.locations("[].speakers[]")
```

### Wildcard Access

When `[]` receives a wildcard selector, it returns an array of nodes instead of a single node. `[]` matches every item of a sequence and `*` matches every value of a map:

```ruby
document["[].title"]       # => [Yerba::Scalar, Yerba::Scalar, ...]
document["[].speakers[]"]  # => [Yerba::Scalar, Yerba::Scalar, ...]
document["items[].name"]   # => [Yerba::Scalar, Yerba::Scalar, ...]

document["database.*"]     # => [Yerba::Scalar, Yerba::Scalar, ...]
document["*"]              # => every value of the root map
document["[].*"]           # => every value of every item

document["[].title"].each { |scalar| puts scalar.value }
document["[].title"].each { |scalar| scalar.value = "Updated" }
```

Each node knows the key it belongs to, which is how a map value can be traced back to its name:

```ruby
document["database.*"].map { |node| [node.key.value, node.value] }
# => [["host", "localhost"], ["port", 5432]]
```

`get_all` resolves the same selectors in a single walk of the document, rather than looking each node up separately. It is the faster choice when reading many nodes, and returns nodes that are still connected to the document:

```ruby
document.get_all("[].title")     # => [Yerba::Scalar, Yerba::Scalar, ...]
document.get_all("database.*")   # => [Yerba::Scalar, Yerba::Scalar, ...]

document.get_all("[].title").first.value = "Updated"
document.save!
```

Reading values rather than nodes keeps a positional `nil` where a branch does not match, while resolving nodes skips it:

```ruby
document.value_at("*.city")            # => [nil, "Berlin", nil]
document.get_all("*.city").map(&:value) # => ["Berlin"]
```

Wildcards address more than one node, so they cannot be written through. `set`, `insert` and `delete` raise rather than change every match at once:

```ruby
document.set("database.*", "x")  # => raises Yerba::Error
document.delete("database.*")    # => raises Yerba::Error
```

### Collections

Operate on multiple files matching a glob pattern:

```ruby
collection = Yerba.files("data/**/videos.yml")

collection.each do |document|
  puts document[0]["title"].value
end

collection.find_by(name: "Alice")
collection.where(kind: "talk")
collection.pluck(:name)

collection.apply! do |document|
  document.set("status", "published")
end
```

Use `Collection.get` to retrieve nodes across all matching files in parallel. Returns `Scalar`, `Map`, or `Sequence` objects with `file_path`, `line`, and `selector`:

```ruby
speakers = Yerba::Collection.get("data/**/videos.yml", "[].speakers[]")

speakers.each do |scalar|
  puts "#{scalar.value} in #{scalar.file_path}:#{scalar.line}"
end

maps = Yerba::Collection.get("data/**/videos.yml", "[]")
maps.first.class
# => Yerba::Map

sequences = Yerba::Collection.get("data/**/videos.yml", "[].speakers")
sequences.first.class
# => Yerba::Sequence
```

Nodes returned by `Collection.get` lazily load their `Document` on first mutation, so reads are fast and writes work transparently:

```ruby
scalars = Yerba::Collection.get("data/**/videos.yml", "[].title")
scalars.first.value = "New Title"
scalars.first.document.save!
```

### Saving

Write changes back to the original file:

```ruby
document.save!
```

Save to a new path:

```ruby
document.save_to!("output.yml")
```

Or render the document as a string without writing to disk:

```ruby
document.to_s
```

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to set up the repo locally, run tests, and contribute.

## License

The gem is available as open source under the terms of the [MIT License](https://github.com/marcoroth/yerba/blob/main/LICENSE.txt).
