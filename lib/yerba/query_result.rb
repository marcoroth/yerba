# frozen_string_literal: true

module Yerba
  class QueryResult
    include Enumerable #[untyped]

    # @rbs @sequence: Sequence
    # @rbs @indices: Array[Integer]

    #: (Sequence sequence, Array[Integer] indices) -> void
    def initialize(sequence, indices)
      @sequence = sequence
      @indices = indices
    end

    #: () { (untyped) -> void } -> Array[Integer]
    #: () -> Enumerator[untyped, Array[Integer]]
    def each
      return enum_for(:each) unless block_given?

      @indices.each { |index| yield @sequence[index] }
    end

    #: (Integer position) -> untyped
    def [](position)
      index = @indices[position]

      @sequence[index] if index
    end

    #: () -> untyped
    def first
      self[0]
    end

    #: () -> untyped
    def last
      self[-1]
    end

    #: () -> Integer
    def length
      @indices.length
    end
    alias size length
    alias count length

    #: () -> bool
    def empty?
      @indices.empty?
    end

    #: () -> Array[Integer]
    def indices
      @indices.dup
    end

    #: () -> String
    def inspect
      "#<Yerba::QueryResult length=#{length}>"
    end
  end
end
