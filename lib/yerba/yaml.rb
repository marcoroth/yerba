# frozen_string_literal: true

require "yaml"

require_relative "../yerba"

module Yerba
  # Routes ::YAML's loaders through Yerba instead of Psych.
  #
  #   require "yerba/yaml"
  #
  #   YAML.load_file("config.yml")   # parsed by Yerba, not Psych
  #
  # This adds no methods of its own. It only replaces names Psych already
  # defines, and only the ones Yerba can back faithfully — the six loaders. Every
  # other Psych method is left alone, either because Yerba has no equivalent or
  # because using Yerba would be wrong; see NOT_REPLACED.
  #
  # Yerba's reader is not bug-compatible with Psych. It returns anchors and
  # aliases verbatim ("*base") and leaves `<<` as a literal key instead of
  # merging, reads dates and timestamps as Strings rather than Date/Time, reads
  # only the first document of a multi-document stream, and does not resolve
  # tags. `permitted_classes`, `permitted_symbols`, `aliases`, `strict_integer`
  # and `parse_symbols` are accepted and ignored, so existing call sites keep
  # working instead of raising ArgumentError.
  #
  # Call `Yerba::YAML.restore!` to hand the loaders back to Psych, and
  # `Yerba::YAML.takeover!` to take them again.
  module YAML
    # Psych methods deliberately left on Psych.
    #
    #   dump, safe_dump, dump_stream  Yerba's writer mangles embedded newlines:
    #                                 {"k" => "a\nb"} emits a double-quoted
    #                                 scalar broken across lines, which YAML
    #                                 folds back to "a b". Silent corruption.
    #   load_stream, parse_stream     Yerba reads only the first document of a
    #                                 multi-document stream.
    #   parse, parse_file             Contract is a Psych::Nodes::Document tree.
    #                                 Yerba's CST is a different shape entirely.
    #   to_json                       Psych-specific visitor.
    #
    # The tag registry (add_builtin_type, add_domain_type, add_tag, remove_type,
    # dump_tags, load_tags, config, libyaml_version, parser) is untouched for the
    # same reason: Yerba does not resolve tags.
    NOT_REPLACED = [
      :dump, :safe_dump, :dump_stream, :load_stream, :parse, :parse_file, :parse_stream, :to_json
    ].freeze

    OVERRIDE = :yerba_yaml_active

    class << self
      def takeover!(&)
        toggle(true, &)
      end

      def restore!(&)
        toggle(false, &)
      end

      def active?
        return false unless ::YAML.singleton_class.include?(Loaders)

        override = Thread.current[OVERRIDE]

        override.nil? ? Loaders.enabled == true : override
      end

      private

      def toggle(state)
        ::YAML.singleton_class.prepend(Loaders) unless ::YAML.singleton_class.include?(Loaders)

        unless block_given?
          Loaders.enabled = state

          return true
        end

        previous = Thread.current[OVERRIDE]
        Thread.current[OVERRIDE] = state

        begin
          yield
        ensure
          Thread.current[OVERRIDE] = previous
        end
      end
    end

    module Reader
      def self.parse(filename)
        yield
      rescue Yerba::ParseError => e
        line, column = e.message.scan(/line (\d+), column (\d+)/).first

        raise Psych::SyntaxError.new(filename, line.to_i, column.to_i, 0, e.message, nil)
      end

      def self.value(document, fallback:, symbolize_names:, freeze:)
        value = document.value_at(Yerba::Document::ROOT_SELECTOR)

        return fallback if value.nil?

        value = symbolize(value) if symbolize_names
        value = deep_freeze(value) if freeze

        value
      end

      def self.symbolize(value)
        case value
        when Hash then value.to_h { |key, nested| [key.is_a?(String) ? key.to_sym : key, symbolize(nested)] }
        when Array then value.map { |item| symbolize(item) }
        else value
        end
      end

      def self.deep_freeze(value)
        case value
        when Hash
          value.each do |key, nested|
            key.freeze
            deep_freeze(nested)
          end
        when Array
          value.each { |item| deep_freeze(item) }
        end

        value.freeze
      end
    end

    module Loaders
      class << self
        attr_accessor :enabled
      end

      def load(yaml, filename: nil, fallback: nil, symbolize_names: false, freeze: false, **)
        return super unless Yerba::YAML.active?

        document = Reader.parse(filename) { Yerba.parse(yaml) }

        Reader.value(document, fallback: fallback, symbolize_names: symbolize_names, freeze: freeze)
      end

      def load_file(filename, fallback: nil, symbolize_names: false, freeze: false, **)
        return super unless Yerba::YAML.active?

        document = Reader.parse(filename) { Yerba.document(filename) }

        Reader.value(document, fallback: fallback, symbolize_names: symbolize_names, freeze: freeze)
      end

      # Yerba never instantiates arbitrary Ruby classes, so safe and unsafe
      # loading are the same operation here. `unsafe_load` keeps Psych's odd
      # `fallback: false` default.
      def safe_load(yaml, **)
        return super unless Yerba::YAML.active?

        load(yaml, **)
      end

      def safe_load_file(filename, **)
        return super unless Yerba::YAML.active?

        load_file(filename, **)
      end

      def unsafe_load(yaml, fallback: false, **)
        return super unless Yerba::YAML.active?

        load(yaml, fallback: fallback, **)
      end

      def unsafe_load_file(filename, fallback: false, **)
        return super unless Yerba::YAML.active?

        load_file(filename, fallback: fallback, **)
      end
    end
  end
end

Yerba::YAML.takeover!
