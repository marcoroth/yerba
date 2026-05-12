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
      value_at(ROOT_SELECTOR)
    end

    def to_a
      value_at(ROOT_SELECTOR)
    end

    def to_yaml
      to_s
    end

    def dig(*keys)
      keys.reduce(self) { |node, key| node.nil? ? nil : node[key] }
    end

    def fetch(selector)
      validate_selector!(selector)

      self[selector]
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

    def inspect
      if path
        "#<Yerba::Document path=#{path.inspect}>"
      else
        "#<Yerba::Document (parsed)>"
      end
    end
  end
end
