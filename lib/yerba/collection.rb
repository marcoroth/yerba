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

    def apply!
      each do |document|
        yield document

        document.save!
      end
    end
  end
end
