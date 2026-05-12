# frozen_string_literal: true

module Yerba
  class Document
    ROOT_SELECTOR = ""

    def self.cache
      @cache ||= Hash.new { |hash, path| hash[path] = new(path) }
    end

    def self.clear_cache!
      @cache = nil
    end

    def selector
      ROOT_SELECTOR
    end

    def root
      self[ROOT_SELECTOR]
    end

    def map?
      root.is_a?(Map)
    end

    def sequence?
      root.is_a?(Sequence)
    end

    def to_h
      get_value(ROOT_SELECTOR)
    end

    def to_a
      get_value(ROOT_SELECTOR)
    end

    def to_yaml
      to_s
    end

    def dig(*keys)
      result = keys.reduce(self) { |node, key| node.nil? ? nil : node[key] }

      result&.value
    end

    def at_path(path)
      return self[path] unless path.include?("[]")

      resolve_selectors(path).filter_map { |selector| self[selector] }
    end

    def locations(selector)
      resolve_selectors(selector).filter_map { |selector| location(selector) }
    end

    def find_by(...)
      root.find_by(...)
    end

    def where(...)
      root.where(...)
    end

    def pluck(...)
      root.pluck(...)
    end

    def <<(item)
      root << item
    end

    def concat(items)
      root.concat(items)
    end

    def save!(apply: false)
      Yerbafile.apply!(self, apply) if apply
      write!

      self
    end

    def apply!(yerbafile = nil)
      apply(yerbafile)
      write! if changed?

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

    def inspect
      if path
        "#<Yerba::Document path=#{path.inspect}>"
      else
        "#<Yerba::Document (parsed)>"
      end
    end
  end
end
