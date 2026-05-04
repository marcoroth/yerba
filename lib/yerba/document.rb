# frozen_string_literal: true

module Yerba
  class Document
    ROOT_SELECTOR = "".freeze

    def root
      self[ROOT_SELECTOR]
    end

    def map?
      root.is_a?(Map)
    end

    def sequence?
      root.is_a?(Sequence)
    end

    def to_h
      get_value(ROOT_SELECTOR)
    end

    def to_a
      get_value(ROOT_SELECTOR)
    end

    def to_yaml
      to_s
    end

    def dig(*keys)
      result = keys.reduce(self) { |node, key| node.nil? ? nil : node[key] }

      result&.value
    end

    def at_path(path)
      if path.include?("[]")
        values = get(path)
        return [] unless values.is_a?(Array)

        path.sub("[]", "")

        values.each_with_index.map do |_value, index|
          resolved_path = path.sub("[]", "[#{index}]")
          self[resolved_path]
        end
      else
        self[path]
      end
    end

    def find_by(...)
      root.find_by(...)
    end

    def where(...)
      root.where(...)
    end

    def pluck(...)
      root.pluck(...)
    end

    def <<(item)
      root << item
    end

    def concat(items)
      root.concat(items)
    end

    def save!(apply: false)
      Yerbafile.apply!(self, apply) if apply
      write!

      self
    end

    def apply!(yerbafile = nil)
      apply(yerbafile)
      write! if changed?

      self
    end

    def apply(yerbafile = nil)
      Yerbafile.apply!(self, yerbafile)

      self
    end

    def inspect
      if path
        "#<Yerba::Document path=#{path.inspect}>"
      else
        "#<Yerba::Document (parsed)>"
      end
    end
  end
end
