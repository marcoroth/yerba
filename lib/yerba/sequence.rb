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

    def concat(items)
      if @document
        hashes = items.map do |item|
          case item
          when Map then item.to_hash
          when Hash then item
          else { value: item.to_s }
          end
        end

        @document.insert_objects(@selector, hashes)
      else
        @data.concat(items)
      end

      self
    end

    def each
      return enum_for(:each) unless block_given?

      length.times { |index| yield self[index] }
    end

    def find_by(selector = nil, value = nil, **criteria)
      index = index_of(selector, value, **criteria)

      self[index] if index
    end

    def where(selector = nil, value = nil, **criteria)
      QueryResult.new(self, indices_of(selector, value, **criteria))
    end

    def pluck(*fields)
      return [] unless @document

      all_values = @document.get_value(@selector)
      return [] unless all_values.is_a?(Array)

      if fields.length == 1

        all_values.map { |item| item.is_a?(Hash) ? item[fields.first.to_s] : item }
      else

        all_values.map { |item| fields.map(&:to_s).map { |field| item.is_a?(Hash) ? item[field] : nil } }
      end
    end

    def index_of(selector = nil, value = nil, **criteria)
      if selector && value.nil? && criteria.empty?
        values = @document&.get("#{@selector}[]")

        return values.index(selector) if values.is_a?(Array)

        return nil
      end

      criteria[selector] = value if selector && value
      pairs = expand_nested_criteria(criteria)

      indices = nil

      pairs.each do |field, expected|
        field_string = field.to_s

        if field_string.include?("[]")
          matching = nested_indices_for(field_string, expected)
        else
          values = @document&.get("#{@selector}[].#{field_string}")

          next unless values.is_a?(Array)

          matching = values.each_with_index.filter_map { |actual, index| index if actual == expected }
        end

        indices = indices ? indices & matching : matching
      end

      indices&.first
    end

    def indices_of(selector = nil, value = nil, **criteria)
      if selector && value.nil? && criteria.empty?
        values = @document&.get("#{@selector}[]")

        if values.is_a?(Array)
          return values.each_with_index.filter_map { |actual, index| index if actual == selector }
        end

        return []
      end

      criteria[selector] = value if selector && value
      pairs = expand_nested_criteria(criteria)

      indices = nil

      pairs.each do |field, expected|
        field_string = field.to_s

        if field_string.include?("[]")
          matching = nested_indices_for(field_string, expected)
        else
          values = @document&.get("#{@selector}[].#{field_string}")

          next unless values.is_a?(Array)

          matching = values.each_with_index.filter_map { |actual, index| index if actual == expected }
        end

        indices = indices ? indices & matching : matching
      end

      indices || []
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

    def expand_nested_criteria(criteria)
      expanded = []

      criteria.each do |field, value|
        if value.is_a?(Hash)
          flatten_hash("#{field}[]", value).each do |path, leaf_value|
            expanded << [path, leaf_value]
          end
        elsif value.is_a?(Array)
          value.each do |item|
            expanded << ["#{field}[]", item]
          end
        else
          expanded << [field, value]
        end
      end

      expanded
    end

    def flatten_hash(prefix, hash)
      result = {}

      hash.each do |key, value|
        path = "#{prefix}.#{key}"

        if value.is_a?(Hash)
          flatten_hash("#{path}[]", value).each { |nested_path, leaf| result[nested_path] = leaf }
        else
          result[path] = value
        end
      end

      result
    end

    def nested_indices_for(field, expected)
      all_values = @document&.get_value(@selector)
      return [] unless all_values.is_a?(Array)

      all_values.each_with_index.filter_map do |item, index|
        next unless item.is_a?(Hash)

        nested_values = dig_values(item, field)
        index if nested_values.include?(expected)
      end
    end

    def dig_values(hash, path)
      parts = path.split(".")
      current = [hash]

      parts.each do |part|
        next_values = []

        current.each do |value|
          if part == "[]" && value.is_a?(Array)
            next_values.concat(value)
          elsif part.end_with?("[]")
            key = part.chomp("[]")
            child = value.is_a?(Hash) ? value[key] : nil
            next_values.concat(child) if child.is_a?(Array)
          elsif value.is_a?(Hash)
            next_values << value[part] if value.key?(part)
          end
        end

        current = next_values
      end

      current
    end

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
