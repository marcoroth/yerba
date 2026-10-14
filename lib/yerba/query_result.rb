# frozen_string_literal: true

module Yerba
  class QueryResult
    include Enumerable

    def initialize(sequence, indices, &query)
      @sequence = sequence
      @indices = indices
      @query = query
      @tree = current_tree
    end

    def each
      return enum_for(:each) unless block_given?

      current_indices.each { |index| yield @sequence[index] }
    end

    def [](position)
      index = current_indices[position]

      @sequence[index] if index
    end

    def first(count = nil)
      return self[0] if count.nil?

      current_indices.first(count).map { |index| @sequence[index] }
    end

    def last(count = nil)
      return self[-1] if count.nil?

      current_indices.last(count).map { |index| @sequence[index] }
    end

    def length
      current_indices.length
    end
    alias size length
    alias count length

    def empty?
      current_indices.empty?
    end

    def indices
      current_indices.dup
    end

    def inspect
      "#<Yerba::QueryResult length=#{length}>"
    end

    private

    def current_indices
      tree = current_tree

      if @query && !tree.equal?(@tree)
        @indices = @query.call
        @tree = tree
      end

      @indices
    end

    def current_tree
      @sequence&.document&.nodes
    end
  end
end
