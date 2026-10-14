# frozen_string_literal: true

require "json"

module Yerba
  # A parsed selector, split into its segments.
  #
  #   Yerba::Selector.parse("[3].talks[1]").segments  # => [3, "talks", 1]
  #   Yerba::Selector.parse("[].speakers[]").segments # => [:all_items, "speakers", :all_items]
  #   Yerba::Selector.parse("database.*").segments    # => ["database", :all_keys]
  #   Yerba::Selector.parse(".title").relative?       # => true
  #
  # Keys are Strings, indexes Integers (negative from the end), `[]` is `:all_items` and `*` is `:all_keys`.
  #
  class Selector
    ALL_KEYS = :all_keys
    ALL_ITEMS = :all_items
    CACHE_LIMIT = 10_000

    attr_reader :segments

    def self.parse(selector)
      return selector if selector.is_a?(Selector)

      string = selector.to_s

      cache[string] ||= begin
        cache.clear if cache.size >= CACHE_LIMIT
        parsed = native_parse(string)

        new(parsed["segments"].map { |segment| decode(segment) }, relative: parsed["relative"], source: string)
      end
    end

    # Builds a selector from segments: Strings (keys), Integers (indexes), `:all_items` or `:all_keys`.
    #
    #   Yerba::Selector.build([3, "talks", 1]).to_s # => "[3].talks[1]"
    #
    def self.build(segments, relative: false)
      new(segments, relative: relative)
    end

    def self.clear_cache!
      @cache = nil
    end

    def self.cache
      @cache ||= {}
    end
    private_class_method :cache

    def self.decode(segment)
      case segment
      when Hash
        segment["all"] == "keys" ? ALL_KEYS : ALL_ITEMS
      else
        segment
      end
    end
    private_class_method :decode

    def initialize(segments, relative: false, source: nil)
      @segments = segments.dup.freeze
      @relative = relative
      @source = source

      freeze
    end

    def relative?
      @relative
    end

    def absolute?
      !@relative
    end

    def empty?
      segments.empty?
    end

    def wildcard?
      segments.any? { |segment| [ALL_ITEMS, ALL_KEYS].include?(segment) }
    end

    # "[3].talks[1]" => "[3].talks"
    def parent
      self.class.build(segments[0...-1], relative: relative?)
    end

    # Selector.parse("[3]").join("talks", 1) => "[3].talks[1]"
    def join(*more)
      self.class.build(segments + more, relative: relative?)
    end

    def dig(values)
      raise ArgumentError, "Cannot dig a wildcard selector (#{self}), use Document#find or Collection.find instead" if wildcard?

      segments.reduce(values) do |current, segment|
        case current
        when Array
          segment.is_a?(Integer) ? current[segment] : nil
        when Hash
          segment.is_a?(Integer) ? nil : current[segment]
        end
      end
    end

    def to_s
      @source || self.class.send(:native_build, JSON.generate(encoded_segments), relative?)
    end
    alias to_str to_s

    def ==(other)
      other.is_a?(Selector) && other.segments == segments && other.relative? == relative?
    end
    alias eql? ==

    def hash
      [self.class, segments, relative?].hash
    end

    def inspect
      "#<#{self.class.name} #{to_s.inspect} #{segments.inspect}>"
    end

    private

    def encoded_segments
      segments.map do |segment|
        case segment
        when ALL_ITEMS
          { all: "items" }
        when ALL_KEYS
          { all: "keys" }
        else
          segment
        end
      end
    end
  end
end
