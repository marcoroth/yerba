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
  end
end
