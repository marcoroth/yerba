# frozen_string_literal: true

module Yerba
  class Scalar
    include Node

    def initialize(value = nil, quote_style: nil)
      init_node(nil, nil, nil, nil, nil, nil)

      @value = value
      @quote_style = quote_style
    end

    def value
      live = live_document if @selector
      adopt_current_node(live) if live && !tree_node?

      return @value.is_a?(String) ? @value.dup : @value if tree_node?

      @value ||= document&.value_at(@selector)
    end

    def quote_style
      @quote_style || document&.get_quote_style(@selector)
    end

    def quote_style=(style)
      document&.set_quote_style(@selector, style)

      @quote_style = style
    end

    def value=(new_value)
      document&.set(@selector, new_value)

      @value = new_value
    end
    alias set value=

    def to_s
      value.to_s
    end

    def to_str
      to_s
    end

    def to_i
      value.to_i
    end

    def to_f
      value.to_f
    end

    def ==(other)
      value == other
    end

    def to_yaml
      case value
      when nil then "null"
      when String then Formatting.quote(value, quote_style)
      else value.to_s
      end
    end

    def delete
      document&.delete(@selector)
    end

    def inspect
      if @selector
        "#<Yerba::Scalar selector=#{@selector.inspect} value=#{value.inspect}>"
      else
        "#<Yerba::Scalar value=#{value.inspect} quote_style=#{quote_style.inspect}>"
      end
    end

    protected

    def stored_value
      @value
    end

    private

    def live_document
      return @document if @document

      document if @file_path && Document.cache.key?(@file_path)
    end

    def adopt_current_node(live)
      node = live[@selector]

      if node.is_a?(Scalar) && node.tree_node?
        @value = node.stored_value
        attach_tree(live.nodes, nil, node.valueless?)
      else
        @value = nil
        @tree = nil
      end
    end

    def init_from(value: nil, quote_style: nil, **)
      @value = value
      @quote_style = quote_style
    end
  end
end
