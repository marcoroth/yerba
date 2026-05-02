# frozen_string_literal: true

module Yerba
  class Sequence
    include Enumerable

    attr_reader :path

    # Two construction modes:
    #   Bound:      Sequence.new(document, "path")  — from document["key"]
    #   Standalone: Sequence.new(["ruby", "rust"])  — for insertion
    def initialize(document_or_array = nil, path = nil)
      if document_or_array.is_a?(Document)
        @document = document_or_array
        @path = path
        @data = nil
      elsif document_or_array.is_a?(Array)
        @document = nil
        @path = nil
        @data = document_or_array
      else
        @document = nil
        @path = nil
        @data = []
      end
    end

    def [](index)
      if @document
        new_path = "#{@path}[#{index}]"
        @document[new_path]
      else
        @data[index]
      end
    end

    def <<(item)
      if @document
        case item
        when Map
          @document.insert_object(@path, item.to_hash)
        when Hash
          @document.insert_object(@path, item)
        when Scalar
          @document.insert(@path, item.to_yaml)
        else
          @document.insert(@path, item.to_s)
        end
      else
        @data << item
      end

      self
    end

    def each
      return enum_for(:each) unless block_given?

      length.times { |i| yield self[i] }
    end

    def length
      if @document
        scalar_items = @document.get("#{@path}[]")

        if scalar_items.is_a?(Array) && !scalar_items.empty?
          scalar_items.length
        else
          parsed = ::YAML.safe_load(@document.to_s)
          data = @path.empty? ? parsed : parsed.dig(*@path.split("."))
          data.is_a?(Array) ? data.length : 0
        end
      else
        @data.length
      end
    end
    alias size length

    def first
      self[0]
    end

    def last
      self[length - 1] # rubocop:disable Style/NegativeArrayIndex
    end

    def delete
      @document&.delete(@path)
    end

    def exists?
      @document ? @document.exists?(@path) : !@data.nil?
    end

    def value
      items
    end

    def to_a
      @data || items
    end
    alias to_ary to_a

    def to_yaml
      items.map do |item|
        case item
        when Scalar then "- #{item.to_yaml}"
        when Map then "- #{item.to_yaml.gsub("\n", "\n  ")}"
        else "- #{item}"
        end
      end.join("\n")
    end

    def inspect
      list = items
      preview = list.first(5).map(&:inspect).join(", ")
      suffix = list.length > 5 ? ", ... (#{list.length} items)" : ""

      if @path
        "#<Yerba::Sequence path=#{@path.inspect} [#{preview}#{suffix}]>"
      else
        "#<Yerba::Sequence [#{preview}#{suffix}]>"
      end
    end

    private

    def items
      if @document
        result = @document.get("#{@path}[]")

        if result.is_a?(Array) && !result.empty?
          result
        else
          parsed = ::YAML.safe_load(@document.to_s)
          data = @path.empty? ? parsed : parsed.dig(*@path.split("."))
          data.is_a?(Array) ? data : []
        end
      else
        @data
      end
    end
  end
end
