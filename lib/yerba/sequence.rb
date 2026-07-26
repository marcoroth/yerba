# frozen_string_literal: true

module Yerba
  class Sequence
    include Enumerable
    include Node

    def initialize(array = nil)
      init_node(nil, nil, nil, nil, nil, nil)

      @data = array.is_a?(Array) ? array : []
    end

    def [](index)
      if connected?
        new_path = "#{@selector}[#{index}]"
        document[new_path]
      else
        @data[index]
      end
    end

    def fetch(index)
      if connected?
        new_path = "#{@selector}[#{index}]"
        result = document[new_path]

        if result.nil?
          raise IndexError, "index #{index} outside of sequence bounds: 0...#{length}"
        end

        result
      else
        @data.fetch(index)
      end
    end

    def dig(*keys)
      if connected?
        keys.reduce(self) { |node, key| node.nil? ? nil : node[key] }
      else
        @data.dig(*keys)
      end
    end

    def value_at(index)
      if connected?
        new_path = "#{@selector}[#{index}]"
        document.value_at(new_path)
      else
        @data[index]
      end
    end

    def <<(item)
      if connected?
        case item
        when Map
          document.insert_object(@selector, item.to_hash)
        when Hash
          document.insert_object(@selector, item)
        when Scalar
          document.insert(@selector, item.to_yaml)
        else
          formatted = format_for_insert(item.to_s)
          document.insert(@selector, formatted)
        end
      else
        @data << item
      end

      self
    end

    def concat(items)
      if connected?
        hashes = items.map do |item|
          case item
          when Map then item.to_hash
          when Hash then item
          else { value: item.to_s }
          end
        end

        document.insert_objects(@selector, hashes)
      else
        @data.concat(items)
      end

      self
    end

    def each(&)
      return enum_for(:each) unless block_given?

      if connected?
        items = document.get_all("#{@selector}[]")

        if items.length == length
          items.each(&)

          return self
        end
      end

      length.times { |index| yield self[index] }

      self
    end

    def find_by(selector = nil, value = nil, **criteria)
      index = index_of(selector, value, **criteria)

      self[index] if index
    end

    def where(selector = nil, value = nil, **criteria)
      QueryResult.new(self, indices_of(selector, value, **criteria))
    end

    def pluck(*fields)
      return [] unless connected?

      all_values = document.value_at(@selector)
      return [] unless all_values.is_a?(Array)

      if fields.length == 1

        all_values.map { |item| item.is_a?(Hash) ? item[fields.first.to_s] : item }
      else

        all_values.map { |item| fields.map(&:to_s).map { |field| item.is_a?(Hash) ? item[field] : nil } }
      end
    end

    def index_of(selector = nil, value = nil, **criteria)
      if selector && value.nil? && criteria.empty?
        all_values = document&.value_at(@selector)

        return nil unless all_values.is_a?(Array)

        return all_values.index(selector.to_s)
      end

      criteria[selector] = value if selector && value
      pairs = expand_nested_criteria(criteria)

      indices = nil

      pairs.each do |field, expected|
        field_string = field.to_s

        if field_string.include?("[]")
          matching = nested_indices_for(field_string, expected)
        else
          all_values = document&.value_at(@selector)

          next unless all_values.is_a?(Array)

          matching = all_values.each_with_index.filter_map do |item, index|
            next unless item.is_a?(Hash)

            actual = dig_into(item, field_string)
            index if actual.to_s == expected.to_s
          end
        end

        indices = indices ? indices & matching : matching
      end

      indices&.first
    end

    def indices_of(selector = nil, value = nil, **criteria)
      if selector && value.nil? && criteria.empty?
        all_values = document&.value_at(@selector)

        if all_values.is_a?(Array)
          return all_values.each_with_index.filter_map { |actual, index| index if actual.to_s == selector.to_s }
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
          all_values = document&.value_at(@selector)

          next unless all_values.is_a?(Array)

          matching = all_values.each_with_index.filter_map do |item, index|
            next unless item.is_a?(Hash)

            actual = dig_into(item, field_string)
            index if actual.to_s == expected.to_s
          end
        end

        indices = indices ? indices & matching : matching
      end

      indices || []
    end

    def length
      if connected?
        scalar_items = document.value_at("#{@selector}[]")

        if scalar_items.is_a?(Array) && !scalar_items.empty?
          scalar_items.length
        else
          data = document.value_at(@selector)

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
      document&.remove(@selector, value.to_s)

      self
    end

    def delete_at(index)
      document&.remove_at(@selector, index)

      self
    end

    def delete_if
      return enum_for(:delete_if) unless block_given?

      indices_to_remove = []

      length.times do |index|
        indices_to_remove << index if yield self[index]
      end

      indices_to_remove.reverse_each do |index|
        document&.remove_at(@selector, index)
      end

      self
    end

    def sort(by: nil, case_sensitive: false)
      document&.sort(@selector, by: by, case_sensitive: case_sensitive)

      self
    end

    # Deletes the entire sequence node from the YAML document.
    #
    # Removes the sequence at its selector path from the parent document.
    # Has no effect if the sequence is not connected to a document.
    #
    # Returns the document, or +nil+ if the sequence is not connected to a document.
    #
    #   document = Yerba::Document.parse(<<~YAML)
    #     items:
    #       - name: Ruby
    #       - name: Rust
    #   YAML
    #
    #   document["items"].delete
    #   document.to_s # => "{}\n"
    def delete
      document&.delete(@selector)
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

    def to_s
      source || super
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

    def collection_style
      @collection_style || document&.get_collection_style(@selector)
    end

    def collection_style=(style)
      document&.set_collection_style(@selector, style)

      @collection_style = style
    end

    def sequence_indent
      @sequence_indent || document&.get_sequence_indent(@selector)
    end

    def sequence_indent=(style)
      document&.set_sequence_indent(@selector, style)

      @sequence_indent = style
    end

    private

    def dig_into(hash, path)
      keys = path.split(".")

      keys.reduce(hash) { |current, key| current.is_a?(Hash) ? current[key] : nil }
    end

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
      all_values = document&.value_at(@selector)
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
      document.get_quote_style("#{@selector}[0]")
    end

    def items
      if connected?
        result = document.value_at("#{@selector}[]")

        if result.is_a?(Array) && !result.empty?
          result
        else
          data = document.value_at(@selector)

          data.is_a?(Array) ? data : []
        end
      else
        @data
      end
    end
  end
end
