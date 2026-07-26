# frozen_string_literal: true

module Yerba
  class Map
    include Enumerable
    include Node

    def initialize(hash = nil, **data)
      init_node(nil, nil, nil, nil, nil, nil)

      @data = if hash.is_a?(Hash)
                hash
              else
                (data.empty? ? {} : data)
              end
    end

    def [](key)
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        document[new_path]
      else
        @data[key]
      end
    end

    def []=(key, value)
      set(key, value)
    end

    def set(key, value, style: nil)
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        coerced = coerce_value(value, style: style)
        is_block_value = coerced.is_a?(String) && (coerced.include?("\n") || coerced.start_with?("- "))

        if document.exists?(new_path) && is_block_value
          # Block-style collections can't replace a scalar via set().
          # Delete the key and re-insert at the same position.
          all_keys = keys
          key_index = all_keys.index(key.to_s)
          after_key = key_index&.positive? ? all_keys[key_index - 1] : nil

          document.delete(new_path)

          if after_key
            document.insert(new_path, coerced, after: after_key)
          else
            document.insert(new_path, coerced)
          end
        elsif document.exists?(new_path)
          document.set(new_path, coerced)
        else
          document.insert(new_path, coerced)
        end
      else
        @data[key] = value
      end
    end

    def insert(key, value, before: nil, after: nil, style: nil)
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"

        document.insert(new_path, coerce_value(value, style: style), before: before, after: after)
      else
        @data[key] = value
      end

      self
    end

    def sort_keys(order)
      document&.sort_keys(@selector, order)

      self
    end

    def keys
      return @data.keys unless connected?

      document.keys_at(@selector)
    end

    def each(&)
      return enum_for(:each) unless block_given?

      unless connected?
        @data.each(&)
        return self
      end

      names = keys
      values = document.get_all(@selector.empty? ? "*" : "#{@selector}.*")

      if values.length == names.length && values.all?(&:key)
        values.each { |value| yield value.key.value, value }
      else
        names.each { |key| yield key, self[key] }
      end

      self
    end

    def fetch(key)
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        document.fetch(new_path)
      else
        @data.fetch(key)
      end
    end

    def dig(*keys)
      if connected?
        keys.reduce(self) { |node, key| node.nil? ? nil : node[key] }
      else
        @data.dig(*keys)
      end
    end

    def value_at(key)
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        document.value_at(new_path)
      else
        @data[key]
      end
    end

    # Deletes a key from the map, or deletes the entire map node from the document.
    #
    # When +key+ is given and the map is connected to a document, deletes that key
    # from the map in the YAML document. When no +key+ is given and the map is
    # connected, removes the entire map node at its selector path. When not connected
    # to a document, delegates to the underlying Hash.
    #
    # Returns +self+.
    #
    #   # Delete a specific key from a connected map:
    #   document = Yerba::Document.parse(<<~YAML)
    #     database:
    #       host: localhost
    #       pool: 10
    #   YAML
    #
    #   document["database"].delete("pool")
    #   document.to_s # => "database:\n  host: localhost\n"
    #
    #   # Delete the entire map node:
    #   document["database"].delete
    #   document.to_s # => "{}\n"
    def delete(key = nil)
      if key && connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        document.delete(new_path)
      elsif connected?
        document.delete(@selector)
      else
        @data.delete(key)
      end

      self
    end

    def key?(key)
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        document.exists?(new_path)
      else
        @data.key?(key)
      end
    end
    alias has_key? key?
    alias include? key?

    def value
      @data || nil
    end

    def to_h
      if connected?
        results = document.find(@selector)
        results&.first || {}
      else
        @data
      end
    end
    alias to_hash to_h

    def to_s
      source || to_yaml
    end

    def to_yaml
      to_hash.map do |key, val|
        formatted = format_value(val)
        "#{key}: #{formatted}"
      end.join("\n")
    end

    def inspect
      if connected?
        results = document.find(@selector)

        if results.is_a?(Array) && !results.empty? && results.first.is_a?(Hash)
          map_keys = results.first.keys.first(5)
          preview = map_keys.map { |key| "#{key}: #{results.first[key].inspect}" }.join(", ")

          "#<Yerba::Map selector=#{@selector.inspect} {#{preview}}>"
        else
          "#<Yerba::Map selector=#{@selector.inspect}>"
        end
      else
        "#<Yerba::Map {#{@data.map { |key, value| "#{key}: #{value.inspect}" }.join(", ")}}>"
      end
    end

    def collection_style
      @collection_style || document&.get_collection_style(@selector)
    end

    def collection_style=(style)
      document&.set_collection_style(@selector, style)

      @collection_style = style
    end

    private

    def format_value(value)
      case value
      when Scalar then value.to_yaml
      when nil then "null"
      else value.to_s
      end
    end

    def coerce_value(value, style: nil)
      case value
      when Array, Hash
        resolved_style = style || :block

        if resolved_style == :flow
          Formatting.to_yaml_value(value)
        else
          Formatting.to_block_yaml_value(value)
        end
      else value
      end
    end
  end
end
