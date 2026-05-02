# frozen_string_literal: true

module Yerba
  class Scalar
    attr_reader :path

    # Two construction modes:
    #   Bound:      Scalar.new(document, "path", value)  — from document["key"]
    #   Standalone: Scalar.new("hello", quote_style: :double)  — for insertion
    def initialize(document_or_value, path_or_opts = nil, value = nil, quote_style: nil)
      if document_or_value.is_a?(Document)
        @document = document_or_value
        @path = path_or_opts
        @value = value
        @quote_style = quote_style
      else
        @document = nil
        @path = nil
        @value = document_or_value
        @quote_style = path_or_opts.is_a?(Hash) ? path_or_opts[:quote_style] : quote_style
      end
    end

    def value
      @value ||= @document&.get(@path)
    end

    def quote_style
      @quote_style || @document&.get_quote_style(@path)
    end

    def quote_style=(style)
      @document&.set_quote_style(@path, style)

      @quote_style = style
    end

    def value=(new_value)
      @document&.set(@path, new_value)
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
      when nil
        "null"
      when String
        case quote_style
        when :double
          "\"#{value.gsub("\\", "\\\\").gsub('"', '\\"')}\""
        when :single
          "'#{value.gsub("'", "''")}'"
        else
          value
        end
      else
        value.to_s
      end
    end

    def delete
      @document&.delete(@path)
    end

    def inspect
      if @path
        "#<Yerba::Scalar path=#{@path.inspect} value=#{value.inspect}>"
      else
        "#<Yerba::Scalar value=#{value.inspect} quote_style=#{quote_style.inspect}>"
      end
    end
  end
end
