# frozen_string_literal: true

module Yerba
  class Document
    include NodeTree

    ROOT_SELECTOR = ""

    def self.cache
      @cache ||= Hash.new { |hash, path| hash[path] = new(path) }
    end

    def self.clear_cache!
      @cache = nil
    end

    def self.from(object, path: nil)
      document = parse(Formatting.to_yaml_document(object))
      document.instance_variable_set(:@path, path) if path
      document
    end

    def selector
      ROOT_SELECTOR
    end

    def root
      self[ROOT_SELECTOR]
    end

    def root=(value)
      replace_content!(Formatting.to_yaml_document(value))
    end

    def []=(key, value)
      if root.is_a?(Scalar)
        raise Error, "document root is not set. Use `document.root = {}` or `document.root = []` first"
      end

      if root.is_a?(Sequence)
        raise Error, "document root is a Sequence, not a Map. Use `document << item` to append, or `document.root = {}` to switch to a Map"
      end

      root[key] = value
    end

    def map?
      root.is_a?(Map)
    end

    def sequence?
      root.is_a?(Sequence)
    end

    def to_h
      value = value_at(ROOT_SELECTOR)
      value.nil? ? {} : value
    end

    def to_a
      value = value_at(ROOT_SELECTOR)
      value.nil? ? [] : value
    end

    def to_yaml
      to_s
    end

    def save_to!(path)
      @path = path
      @loaded_mtime = nil
      save!
    end

    def stale?
      return false unless @path && @loaded_mtime

      File.mtime(@path) != @loaded_mtime
    end

    def nodes
      generation = native_generation
      return @nodes if @nodes_generation == generation

      @nodes_generation = generation
      @nodes = build_nodes(native_node_records)
    end

    def dig(*keys)
      keys.reduce(self) { |node, key| node.nil? ? nil : node[key] }
    end

    def fetch(selector)
      validate_selector!(selector)

      self[selector]
    end

    def find_by(...)
      return nil if empty_root?

      root.find_by(...)
    end

    def where(...)
      return QueryResult.new(nil, []) if empty_root?

      root.where(...)
    end

    def pluck(...)
      return [] if empty_root?

      root.pluck(...)
    end

    def <<(item)
      if root.is_a?(Scalar)
        raise Error, "document root is not set. Use `document.root = []` first"
      end

      if root.is_a?(Map)
        raise Error, "document root is a Map, not a Sequence. Use `document[\"key\"] = value` to set keys, or `document.root = []` to switch to a Sequence"
      end

      root << item
    end

    def concat(items)
      root.concat(items)
    end

    def save!(apply: false)
      check_stale!
      Yerbafile.apply!(self, apply) if apply
      write!
      @loaded_mtime = File.mtime(@path) if @path
      self
    end

    def apply!(yerbafile = nil)
      check_stale!
      apply(yerbafile)
      write! if changed?
      @loaded_mtime = File.mtime(@path) if @path

      self
    end

    def apply(yerbafile = nil)
      Yerbafile.apply!(self, yerbafile)

      self
    end

    def valid?(schema, selector: nil)
      errors = validate(schema, selector: selector)

      errors.empty?
    end

    def validate(schema, selector: nil)
      schema_json = schema.is_a?(String) ? schema : JSON.generate(schema)

      validate_schema(schema_json, selector)
    end

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

    def build_nodes(records)
      built = []

      records.each do |parent, type, selector, key_name, value, has_value, location, key_location|
        key = Scalar.from_document(nil, nil, key_location, nil, value: key_name) if key_name
        node = build_node(type, selector, location, key, value)

        node.attach_tree(built.first || node, tree_container(type), !has_value) if type
        add_child(built[parent], key_name, node) unless parent.negative?

        built << node
      end

      built.each { |node| node&.tree_children&.freeze }
      built.first
    end

    def build_node(type, selector, location, key, value)
      case type
      when :scalar
        Scalar.from_document(self, selector, location, key, value: value)
      when :map
        Map.from_document(self, selector, location, key)
      when :sequence
        Sequence.from_document(self, selector, location, key)
      else
        native_node_at(selector)
      end
    end

    def tree_container(type)
      case type
      when :map
        {}
      when :sequence
        []
      end
    end

    def add_child(parent, key_name, node)
      children = parent.tree_children

      if children.is_a?(Hash)
        children[key_name] = node
      else
        children << node
      end
    end

    def empty_root?
      root.is_a?(Scalar) && root.value.nil?
    end

    def check_stale!
      return unless stale?

      raise Yerba::StaleFileError, "#{@path} was modified externally since it was loaded"
    end

    public

    def inspect
      if path
        "#<Yerba::Document path=#{path.inspect}>"
      else
        "#<Yerba::Document (parsed)>"
      end
    end
  end
end
