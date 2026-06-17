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
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        coerced = coerce_value(value)

        if document.exists?(new_path)
          document.set(new_path, coerced)
        else
          document.insert(new_path, coerced)
        end
      else
        @data[key] = value
      end
    end

    def insert(key, value, before: nil, after: nil)
      if connected?
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"

        document.insert(new_path, coerce_value(value), before: before, after: after)
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
      if connected?
        results = document.find(@selector)
        return [] unless results.is_a?(Array) && results.first.is_a?(Hash)

        results.first.keys
      else
        @data.keys
      end
    end

    def each(&)
      return enum_for(:each) unless block_given?

      if connected?
        keys.each { |key| yield key, self[key] }
      else
        @data.each(&)
      end
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

    private

    def format_value(value)
      case value
      when Scalar then value.to_yaml
      when nil then "null"
      else value.to_s
      end
    end

    def coerce_value(value)
      case value
      when Array, Hash then Formatting.to_yaml_value(value)
      else value
      end
    end
  end
end
