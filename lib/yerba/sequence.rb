# frozen_string_literal: true

module Yerba
  class Sequence
    include Enumerable

    attr_reader :selector, :location, :key

    def initialize(document_or_array = nil, selector = nil, location = nil, key = nil)
      if document_or_array.is_a?(Document)
        @document = document_or_array
        @selector = selector
        @location = location
        @key = key
        @data = nil
      elsif document_or_array.is_a?(Array)
        @document = nil
        @selector = nil
        @location = nil
        @data = document_or_array
      else
        @document = nil
        @selector = nil
        @location = nil
        @data = []
      end
    end

    def [](index)
      if @document
        new_path = "#{@selector}[#{index}]"
        @document[new_path]
      else
        @data[index]
      end
    end

    def <<(item)
      if @document
        case item
        when Map
          @document.insert_object(@selector, item.to_hash)
        when Hash
          @document.insert_object(@selector, item)
        when Scalar
          @document.insert(@selector, item.to_yaml)
        else
          formatted = format_for_insert(item.to_s)
          @document.insert(@selector, formatted)
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
        scalar_items = @document.get("#{@selector}[]")

        if scalar_items.is_a?(Array) && !scalar_items.empty?
          scalar_items.length
        else
          data = @document.get_value(@selector)
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

    def remove(value)
      @document&.remove(@selector, value.to_s)

      self
    end

    def delete_at(index)
      @document&.remove_at(@selector, index)

      self
    end

    def delete_if
      return enum_for(:delete_if) unless block_given?

      indices_to_remove = []

      length.times do |index|
        indices_to_remove << index if yield self[index]
      end

      indices_to_remove.reverse_each do |index|
        @document&.remove_at(@selector, index)
      end

      self
    end

    def sort(by: nil, case_sensitive: false)
      @document&.sort(@selector, by: by, case_sensitive: case_sensitive)

      self
    end

    def delete
      @document&.delete(@selector)
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

      if @selector
        "#<Yerba::Sequence selector=#{@selector.inspect} [#{preview}#{suffix}]>"
      else
        "#<Yerba::Sequence [#{preview}#{suffix}]>"
      end
    end

    private

    def format_for_insert(value)
      Formatting.quote(value, detect_quote_style)
    end

    def detect_quote_style
      @document.get_quote_style("#{@selector}[0]")
    end

    def items
      if @document
        result = @document.get("#{@selector}[]")

        if result.is_a?(Array) && !result.empty?
          result
        else
          data = @document.get_value(@selector)
          data.is_a?(Array) ? data : []
        end
      else
        @data
      end
    end
  end
end
