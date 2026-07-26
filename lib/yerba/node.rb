# frozen_string_literal: true

module Yerba
  module Node
    attr_reader :selector, :location, :key

    def file_path
      @file_path || @document&.path
    end

    def line
      @line || @location&.start_line
    end

    def document
      @document ||= @file_path ? Yerba::Document.cache[@file_path] : nil
    end

    def connected?
      !@document.nil? || !@file_path.nil?
    end

    def source
      return nil unless connected? && @selector

      document.source(@selector)
    end

    module ClassMethods
      def from_document(document, selector, location = nil, key = nil, **attributes)
        instance = allocate

        instance.send(:init_node, document, selector, location, key, nil, nil)
        instance.send(:init_from, **attributes) if instance.respond_to?(:init_from, true)

        instance
      end

      def from(selector:, file_path: nil, line: nil, location: nil, document: nil, key: nil, **attributes)
        instance = allocate

        instance.send(:init_node, document, selector, coerce_location(location), key, file_path, line)
        instance.send(:init_from, **attributes) if instance.respond_to?(:init_from, true)

        instance
      end

      def coerce_location(location)
        return location unless location.is_a?(Hash)

        Location.new(**location.transform_keys(&:to_sym))
      end
    end

    def self.included(base)
      base.extend(ClassMethods)
    end

    private

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
