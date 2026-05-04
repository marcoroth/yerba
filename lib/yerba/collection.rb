# frozen_string_literal: true

module Yerba
  class Collection
    include Enumerable

    def initialize(glob)
      @glob = glob
    end

    def each
      Dir.glob(@glob).each { |path| yield Document.new(path) }
    end

    def get(path)
      self.class.get(@glob, path)
    end

    def find(path, condition: nil, select: nil)
      self.class.find(@glob, path, condition: condition, select: select)
    end

    def find_by(...)
      each do |document|
        next unless document.sequence?

        result = document.root.find_by(...)
        return result if result
      end

      nil
    end

    def where(...)
      results = []

      each do |document|
        next unless document.sequence?

        results.concat(document.root.where(...).to_a)
      end

      results
    end

    def pluck(...)
      results = []

      each do |document|
        next unless document.sequence?

        results.concat(document.root.pluck(...))
      end

      results
    end

    def apply!
      each do |document|
        yield document

        document.save!
      end
    end
  end
end
