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
      else value.to_s
      end
    end

    def self.to_block_yaml_value(value, indent = 0)
      prefix = "  " * indent

      case value
      when Array
        return "[]" if value.empty?

        value.map { |item|
          if item.is_a?(Hash)
            inner = to_block_yaml_value(item, indent + 1)
            "#{prefix}- #{inner.lstrip}"
          else
            "#{prefix}- #{to_scalar_value(item)}"
          end
        }.join("\n")
      when Hash
        return "{}" if value.empty?

        value.map { |key, value|
          if value.is_a?(Hash) || value.is_a?(Array)
            inner = to_block_yaml_value(value, indent + 1)
            "#{prefix}#{key}:\n#{inner}"
          else
            "#{prefix}#{key}: #{to_scalar_value(value)}"
          end
        }.join("\n")
      else
        to_scalar_value(value)
      end
    end

    def self.to_scalar_value(value)
      case value
      when true then "true"
      when false then "false"
      when nil then "null"
      else value.to_s
      end
    end

    def self.to_yaml_document(value)
      case value
      when Array
        value.empty? ? "---\n[]\n" : "---\n#{to_block_yaml_value(value)}\n"
      when Hash
        value.empty? ? "---\n{}\n" : "---\n#{to_block_yaml_value(value)}\n"
      else
        raise ArgumentError, "expected Array or Hash, got #{value.class}"
      end
    end
  end
end
