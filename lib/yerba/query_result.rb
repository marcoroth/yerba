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

    def first(count = nil)
      return self[0] if count.nil?

      @indices.first(count).map { |index| @sequence[index] }
    end

    def last(count = nil)
      return self[-1] if count.nil?

      @indices.last(count).map { |index| @sequence[index] }
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
