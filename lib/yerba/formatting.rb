# frozen_string_literal: true

module Yerba
  module Formatting
    def self.quote(value, style)
      case style
      when :double
        escaped = value.to_s.gsub("\\", "\\\\").gsub('"', '\\"')
        "\"#{escaped}\""
      when :single
        escaped = value.to_s.gsub("'", "''")
        "'#{escaped}'"
      else
        value.to_s
      end
    end

    def self.to_yaml_value(value)
      case value
      when Array
        return "[]" if value.empty?

        items = value.map { |item| to_yaml_value(item) }

        "[#{items.join(", ")}]"
      when Hash
        return "{}" if value.empty?

        pairs = value.map { |key, value| "#{key}: #{to_yaml_value(value)}" }

        "{#{pairs.join(", ")}}"
      when true then "true"
      when false then "false"
      when nil then "null"
      when Numeric then value.to_s
      else value.to_s
      end
    end
  end
end
