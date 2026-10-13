# frozen_string_literal: true

module Yerba
  module NodeTree
    protected

    def tree_node?
      !@tree.nil? && @tree.equal?(document.nodes)
    end

    def tree_children
      @children
    end

    def valueless?
      @valueless == true
    end

    def attach_tree(tree, children, valueless)
      @tree = tree
      @children = children
      @valueless = valueless
    end
  end

  module Node
    include NodeTree

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

    def children
      tree = current_children

      case tree
      when Hash
        tree.transform_values { |child| child&.dup }
      when Array
        tree.map { |child| child&.dup }
      end
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

    def current_children
      node = tree_node? ? self : document&.[](@selector)

      node.tree_children if node.is_a?(Node) && node.tree_node?
    end

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
