# frozen_string_literal: true

module Yerba
  class Map
    include Enumerable

    attr_reader :path

    # Two construction modes:
    #   Bound:      Map.new(document, "path")  — from document["key"]
    #   Standalone: Map.new(host: "localhost", port: 5432)  — for insertion
    def initialize(document_or_hash = nil, path = nil)
      if document_or_hash.is_a?(Document)
        @document = document_or_hash
        @path = path
        @data = nil
      elsif document_or_hash.is_a?(Hash)
        @document = nil
        @path = nil
        @data = document_or_hash
      else
        @document = nil
        @path = nil
        @data = {}
      end
    end

    def [](key)
      if @document
        new_path = @path.empty? ? key.to_s : "#{@path}.#{key}"
        @document[new_path]
      else
        @data[key]
      end
    end

    def []=(key, value)
      if @document
        new_path = @path.empty? ? key.to_s : "#{@path}.#{key}"
        @document.set(new_path, value)
      else
        @data[key] = value
      end
    end

    def keys
      if @document
        results = @document.find(@path)
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

    def value
      @data || nil
    end

    def to_h
      if @document
        results = @document.find(@path)
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
        results = @document.find(@path)

        if results.is_a?(Array) && !results.empty? && results.first.is_a?(Hash)
          map_keys = results.first.keys.first(5)
          preview = map_keys.map { |key| "#{key}: #{results.first[key].inspect}" }.join(", ")

          "#<Yerba::Map path=#{@path.inspect} {#{preview}}>"
        else
          "#<Yerba::Map path=#{@path.inspect}>"
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
