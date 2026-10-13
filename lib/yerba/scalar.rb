# frozen_string_literal: true

module Yerba
  class Scalar
    include Node

    # @rbs!
    #   extend Node::ClassMethods

    # @rbs @value: Yerba::scalar_value
    # @rbs @quote_style: (Yerba::quote_style | String)?

    #: (?Yerba::scalar_value value, ?quote_style: (Yerba::quote_style | String)?) -> void
    def initialize(value = nil, quote_style: nil)
      init_node(nil, nil, nil, nil, nil, nil)

      @value = value
      @quote_style = quote_style
    end

    #: () -> Yerba::scalar_value
    def value
      @value ||= document&.value_at(@selector)
    end

    #: () -> (Yerba::quote_style | String)?
    def quote_style
      @quote_style || document&.get_quote_style(@selector)
    end

    #: (Yerba::quote_style | String style) -> (Yerba::quote_style | String)
    def quote_style=(style)
      document&.set_quote_style(@selector, style)

      @quote_style = style
    end

    #: (Yerba::scalar_value new_value) -> Yerba::scalar_value
    def value=(new_value)
      document&.set(@selector, new_value)

      @value = new_value
    end
    alias set value=

    #: () -> String
    def to_s
      value.to_s
    end

    #: () -> String
    def to_str
      to_s
    end

    #: () -> Integer
    def to_i
      value = self.value #: untyped
      value.to_i
    end

    #: () -> Float
    def to_f
      value = self.value #: untyped
      value.to_f
    end

    #: (untyped other) -> bool
    def ==(other)
      value == other
    end

    #: () -> String
    def to_yaml
      case value
      when nil then "null"
      when String then Formatting.quote(value, quote_style)
      else value.to_s
      end
    end

    #: () -> Document?
    def delete
      document&.delete(@selector)
    end

    #: () -> String
    def inspect
      if @selector
        "#<Yerba::Scalar selector=#{@selector.inspect} value=#{value.inspect}>"
      else
        "#<Yerba::Scalar value=#{value.inspect} quote_style=#{quote_style.inspect}>"
      end
    end

    private

    #: (?value: Yerba::scalar_value, ?quote_style: (Yerba::quote_style | String)?, **untyped) -> void
    def init_from(value: nil, quote_style: nil, **)
      @value = value
      @quote_style = quote_style
    end
  end
end
