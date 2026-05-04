# frozen_string_literal: true

module Yerba
  class QueryResult
    include Enumerable

    def initialize(sequence, indices)
      @sequence = sequence
      @indices = indices
    end

    def each
      return enum_for(:each) unless block_given?

      @indices.each { |index| yield @sequence[index] }
    end

    def [](position)
      index = @indices[position]

      @sequence[index] if index
    end

    def first
      self[0]
    end

    def last
      self[-1]
    end

    def length
      @indices.length
    end
    alias size length
    alias count length

    def empty?
      @indices.empty?
    end

    def indices
      @indices.dup
    end

    def inspect
      "#<Yerba::QueryResult length=#{length}>"
    end
  end
end
