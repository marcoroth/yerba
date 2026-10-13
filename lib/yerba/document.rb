# frozen_string_literal: true

module Yerba
  class Document
    # @rbs!
    #   # A schema validation error, as returned by Document#validate.
    #   type schema_error = Hash[String, String | Integer | nil]
    #
    #   # Methods below are defined in the C extension.
    #
    #   def initialize: (?String? path) -> void
    #   def self.parse: (String content) -> Document
    #   def replace_content!: (String content) -> self
    #
    #   # Returns an Array of nodes for wildcard selectors.
    #   def []: (Integer index) -> Yerba::node?
    #         | (String selector) -> (Yerba::node | Array[Yerba::node] | nil)
    #   def node_at: (Integer index) -> Yerba::node?
    #              | (String selector) -> (Yerba::node | Array[Yerba::node] | nil)
    #   def value_at: (String selector) -> Yerba::value
    #   def values: () -> Yerba::value
    #   def get_all: (String selector) -> Array[Yerba::node]
    #   def location: (?String? selector) -> Location?
    #   def locations: (String selector) -> Array[Location]
    #   def selectors: () -> Array[String]
    #   def keys_at: (String selector) -> Array[String]
    #   def resolve_selectors: (String selector) -> Array[String]
    #
    #   def get_quote_style: (String selector) -> Yerba::quote_style?
    #   def set_quote_style: (String selector, Yerba::quote_style | String style) -> self
    #   def get_collection_style: (String selector) -> Yerba::collection_style?
    #   def set_collection_style: (String selector, Yerba::collection_style | String style) -> self
    #   def get_sequence_indent: (String selector) -> Yerba::sequence_indent?
    #   def set_sequence_indent: (String selector, Yerba::sequence_indent | String style) -> self
    #
    #   def exists?: (String selector) -> bool
    #   def valid_selector?: (String selector) -> bool
    #   def condition?: (String condition, ?path: String?) -> bool
    #   def find: (String selector, ?condition: String?, ?select: String?) -> Array[Yerba::value]
    #
    #   def set: (String selector, Yerba::scalar_value value, ?condition: String?, ?condition_path: String?, ?if_exists: boolish, ?if_missing: boolish, ?all: boolish, ?plain: boolish) -> self
    #   def insert: (String selector, Yerba::scalar_value value, ?before: String?, ?after: String?, ?at: Integer?, ?style: (Yerba::quote_style | String)?, ?plain: boolish) -> self
    #   def insert_object: (String selector, Hash[untyped, untyped] object, ?before: String?, ?after: String?, ?at: Integer?) -> self
    #   def insert_objects: (String selector, Array[untyped] objects) -> self
    #   def delete: (String selector, ?condition: String?, ?condition_path: String?) -> self
    #   def remove: (String selector, String value) -> self
    #   def remove_at: (String selector, Integer index) -> self
    #   def rename: (String source, String destination) -> self
    #   def sort: (?String? selector, ?by: (String | Symbol)?, ?order: (:asc | :desc | String | Array[String])?, ?case_sensitive: boolish) -> self
    #   def sort_keys: (String selector, String | Array[String] order) -> self
    #   def quote_style: (?path: String?, ?key_style: String?, ?value_style: String?) -> self
    #   def blank_lines: (String selector, Integer count) -> self
    #
    #   def apply_yerbafile: (?String? yerbafile_path) -> self
    #   def validate_schema: (String schema_json, ?String? selector) -> Array[schema_error]
    #   def source: (String selector) -> String?
    #   def to_s: () -> String
    #   def write!: () -> self
    #   def changed?: () -> bool
    #   def path: () -> String?

    # @rbs self.@cache: Hash[String, Document]?
    # @rbs @path: String?
    # @rbs @loaded_mtime: Time?

    ROOT_SELECTOR = "" #: String

    #: () -> Hash[String, Document]
    def self.cache
      @cache ||= Hash.new do |hash, path|
        # @type var path: String
        hash[path] = new(path)
      end
    end

    #: () -> void
    def self.clear_cache!
      @cache = nil
    end

    #: (Array[untyped] | Hash[untyped, untyped] object, ?path: String?) -> Document
    def self.from(object, path: nil)
      document = parse(Formatting.to_yaml_document(object))
      document.instance_variable_set(:@path, path) if path
      document
    end

    #: () -> String
    def selector
      ROOT_SELECTOR
    end

    #: () -> Yerba::node
    def root
      self[ROOT_SELECTOR] #: Yerba::node
    end

    #: (Array[untyped] | Hash[untyped, untyped] value) -> void
    def root=(value)
      replace_content!(Formatting.to_yaml_document(value))
    end

    #: (String | Symbol key, untyped value) -> void
    def []=(key, value)
      root = self.root

      if root.is_a?(Scalar)
        raise Error, "document root is not set. Use `document.root = {}` or `document.root = []` first"
      end

      if root.is_a?(Sequence)
        raise Error, "document root is a Sequence, not a Map. Use `document << item` to append, or `document.root = {}` to switch to a Map"
      end

      root[key] = value
    end

    #: () -> bool
    def map?
      root.is_a?(Map)
    end

    #: () -> bool
    def sequence?
      root.is_a?(Sequence)
    end

    #: () -> Yerba::value
    def to_h
      value = value_at(ROOT_SELECTOR)
      value.nil? ? {} : value
    end

    #: () -> Yerba::value
    def to_a
      value = value_at(ROOT_SELECTOR)
      value.nil? ? [] : value
    end

    #: () -> String
    def to_yaml
      to_s
    end

    #: (String path) -> self
    def save_to!(path)
      @path = path
      @loaded_mtime = nil
      save!
    end

    #: () -> bool
    def stale?
      path = @path
      loaded_mtime = @loaded_mtime

      return false unless path && loaded_mtime

      File.mtime(path) != loaded_mtime
    end

    #: (*(String | Integer) keys) -> untyped
    def dig(*keys)
      keys.reduce(self) { |node, key| node.nil? ? nil : node[key] } #$ untyped
    end

    #: (String selector) -> (Yerba::node | Array[Yerba::node] | nil)
    def fetch(selector)
      validate_selector!(selector)

      self[selector]
    end

    #: (?(String | Symbol)? selector, ?untyped value, **untyped criteria) -> untyped
    def find_by(...)
      return nil if empty_root?

      sequence = root #: Sequence
      sequence.find_by(...)
    end

    #: (?(String | Symbol)? selector, ?untyped value, **untyped criteria) -> QueryResult
    def where(...)
      return QueryResult.new(Sequence.new, []) if empty_root?

      sequence = root #: Sequence
      sequence.where(...)
    end

    #: (*(String | Symbol) fields) -> Array[untyped]
    def pluck(...)
      return [] if empty_root?

      sequence = root #: Sequence
      sequence.pluck(...)
    end

    #: (untyped item) -> Sequence
    def <<(item)
      root = self.root

      if root.is_a?(Scalar)
        raise Error, "document root is not set. Use `document.root = []` first"
      end

      if root.is_a?(Map)
        raise Error, "document root is a Map, not a Sequence. Use `document[\"key\"] = value` to set keys, or `document.root = []` to switch to a Sequence"
      end

      root << item
    end

    #: (Array[untyped] items) -> Sequence
    def concat(items)
      sequence = root #: Sequence
      sequence.concat(items)
    end

    #: (?apply: Yerbafile | String | true | false) -> self
    def save!(apply: false)
      check_stale!
      Yerbafile.apply!(self, apply) if apply
      write!
      path = @path
      @loaded_mtime = File.mtime(path) if path
      self
    end

    #: (?(Yerbafile | String | true)? yerbafile) -> self
    def apply!(yerbafile = nil)
      check_stale!
      apply(yerbafile)
      write! if changed?
      path = @path
      @loaded_mtime = File.mtime(path) if path

      self
    end

    #: (?(Yerbafile | String | true)? yerbafile) -> self
    def apply(yerbafile = nil)
      Yerbafile.apply!(self, yerbafile)

      self
    end

    #: (String | Hash[untyped, untyped] schema, ?selector: String?) -> bool
    def valid?(schema, selector: nil)
      errors = validate(schema, selector: selector)

      errors.empty?
    end

    #: (String | Hash[untyped, untyped] schema, ?selector: String?) -> Array[schema_error]
    def validate(schema, selector: nil)
      schema_json = schema.is_a?(String) ? schema : JSON.generate(schema)

      validate_schema(schema_json, selector)
    end

    #: (String selector) -> void
    def validate_selector!(selector)
      return if valid_selector?(selector)

      available = selectors
      message = "selector \"#{selector}\" is not valid for this document"

      if available.any?
        suggestions = DidYouMean::SpellChecker.new(dictionary: available).correct(selector)

        if suggestions.any?
          message += ". Did you mean: #{suggestions.first(3).join(", ")}?"
        else
          message += ". Available selectors: #{available.first(5).join(", ")}"
          message += ", ..." if available.length > 5
        end
      end

      raise Yerba::SelectorNotFoundError, message
    end

    private

    #: () -> bool
    def empty_root?
      root.is_a?(Scalar) && root.value.nil?
    end

    #: () -> void
    def check_stale!
      return unless stale?

      raise Yerba::StaleFileError, "#{@path} was modified externally since it was loaded"
    end

    public

    #: () -> String
    def inspect
      if path
        "#<Yerba::Document path=#{path.inspect}>"
      else
        "#<Yerba::Document (parsed)>"
      end
    end
  end
end
