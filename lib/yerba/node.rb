# frozen_string_literal: true

module Yerba
  module Node
    # @rbs @document: Document?
    # @rbs @location: Location?
    # @rbs @key: Scalar?
    # @rbs @file_path: String?
    # @rbs @line: Integer?

    attr_reader :selector #: String?
    attr_reader :location #: Location?
    attr_reader :key #: Scalar?

    # A node is either connected to a document (it has a selector and a document or a file path)
    # or disconnected (Scalar.new, Map.new, Sequence.new, which keep their value in `@value`/`@data`).
    # Map and Sequence only use `document`, `@selector` and `@data` on the side they've checked with
    # `connected?`, which Steep can't follow, so these are untyped: `document` is a Document when
    # connected and nil otherwise, `@selector` a String when connected and nil otherwise, and
    # `@data` (in Map and Sequence) a Hash/Array when disconnected and nil otherwise.
    #
    # @rbs @selector: untyped

    #: () -> String?
    def file_path
      @file_path || @document&.path
    end

    #: () -> Integer?
    def line
      @line || @location&.start_line
    end

    # The Document this node belongs to, or nil when disconnected (untyped, see above).
    #: () -> untyped
    def document
      @document ||= @file_path ? Yerba::Document.cache[@file_path] : nil
    end

    #: () -> bool
    def connected?
      !@document.nil? || !@file_path.nil?
    end

    #: () -> String?
    def source
      return nil unless connected? && @selector

      document.source(@selector)
    end

    # Extended into Scalar, Map and Sequence.
    # @rbs module-self Class
    module ClassMethods
      #: (Document? document, String? selector, ?Location? location, ?Scalar? key, **untyped attributes) -> Yerba::node
      def from_document(document, selector, location = nil, key = nil, **attributes)
        instance = allocate

        instance.send(:init_node, document, selector, location, key, nil, nil)
        instance.send(:init_from, **attributes) if instance.respond_to?(:init_from, true)

        instance
      end

      #: (selector: String?, ?file_path: String?, ?line: Integer?, ?location: Location | Hash[untyped, untyped] | nil, ?document: Document?, ?key: Scalar?, **untyped attributes) -> Yerba::node
      def from(selector:, file_path: nil, line: nil, location: nil, document: nil, key: nil, **attributes)
        instance = allocate

        instance.send(:init_node, document, selector, coerce_location(location), key, file_path, line)
        instance.send(:init_from, **attributes) if instance.respond_to?(:init_from, true)

        instance
      end

      #: (Location | Hash[untyped, untyped] | nil location) -> Location?
      def coerce_location(location)
        return location unless location.is_a?(Hash)

        Location.new(**location.transform_keys(&:to_sym)) # steep:ignore
      end
    end

    #: (Module base) -> void
    def self.included(base)
      base.extend(ClassMethods)
    end

    private

    #: (Document? document, String? selector, Location? location, Scalar? key, String? file_path, Integer? line) -> void
    def init_node(document, selector, location, key, file_path, line)
      @document = document
      @selector = selector
      @location = location
      @key = key
      @file_path = file_path
      @line = line
    end
  end
end
