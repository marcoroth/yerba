# frozen_string_literal: true

module Yerba
  class Map
    include Enumerable

    attr_reader :selector, :location, :key

    def initialize(document_or_hash = nil, selector = nil, location = nil, key = nil)
      if document_or_hash.is_a?(Document)
        @document = document_or_hash
        @selector = selector
        @location = location
        @key = key
        @data = nil
      elsif document_or_hash.is_a?(Hash)
        @document = nil
        @selector = nil
        @location = nil
        @data = document_or_hash
      else
        @document = nil
        @selector = nil
        @location = nil
        @data = {}
      end
    end

    def [](key)
      if @document
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        @document[new_path]
      else
        @data[key]
      end
    end

    def []=(key, value)
      if @document
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        @document.set(new_path, value)
      else
        @data[key] = value
      end
    end

    def insert(key, value, before: nil, after: nil)
      if @document
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        @document.insert(new_path, value.to_s, before: before, after: after)
      else
        @data[key] = value
      end

      self
    end

    def sort_keys(order)
      @document&.sort_keys(@selector, order)

      self
    end

    def keys
      if @document
        results = @document.find(@selector)
        return [] unless results.is_a?(Array) && results.first.is_a?(Hash)

        results.first.keys
      else
        @data.keys
      end
    end

    def each(&)
      return enum_for(:each) unless block_given?

      if @document
        keys.each { |key| yield key, self[key] }
      else
        @data.each(&)
      end
    end

    def dig(*keys)
      if @document
        result = keys.reduce(self) { |node, key| node.nil? ? nil : node[key] }
        result&.value
      else
        @data.dig(*keys)
      end
    end

    def delete(key = nil)
      if key && @document
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        @document.delete(new_path)
      elsif @document
        @document.delete(@selector)
      else
        @data.delete(key)
      end

      self
    end

    def key?(key)
      if @document
        new_path = @selector.empty? ? key.to_s : "#{@selector}.#{key}"
        @document.exists?(new_path)
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
      if @document
        results = @document.find(@selector)
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
      if @document
        results = @document.find(@selector)

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
  end
end
