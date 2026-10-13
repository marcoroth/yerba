# frozen_string_literal: true

module Yerba
  class Collection
    include Enumerable #[Document]

    # @rbs!
    #   # Defined in the C extension.
    #   def self.get: (String glob, String selector) -> Array[Yerba::node]
    #
    #   # Defined in the C extension. Each result carries the matching file in "__file".
    #   def self.find: (String glob, String selector, ?condition: String?, ?select: String?) -> Array[Hash[String, Yerba::value]]

    # @rbs @glob: String

    #: (String glob) -> void
    def initialize(glob)
      @glob = glob
    end

    #: () { (Document) -> void } -> void
    def each
      Dir.glob(@glob).each { |path| yield Document.new(path) }
    end

    #: (String path) -> Array[Yerba::node]
    def get(path)
      self.class.get(@glob, path)
    end

    #: (String path, ?condition: String?, ?select: String?) -> Array[Hash[String, Yerba::value]]
    def find(path, condition: nil, select: nil)
      self.class.find(@glob, path, condition: condition, select: select)
    end

    #: (?(String | Symbol)? selector, ?untyped value, **untyped criteria) -> untyped
    def find_by(...)
      each do |document|
        next unless document.sequence?

        sequence = document.root #: Sequence
        result = sequence.find_by(...)
        return result if result
      end

      nil
    end

    #: (?(String | Symbol)? selector, ?untyped value, **untyped criteria) -> Array[untyped]
    def where(...)
      results = [] #: Array[untyped]

      each do |document|
        next unless document.sequence?

        sequence = document.root #: Sequence
        results.concat(sequence.where(...).to_a)
      end

      results
    end

    #: (*(String | Symbol) fields) -> Array[untyped]
    def pluck(...)
      results = [] #: Array[untyped]

      each do |document|
        next unless document.sequence?

        sequence = document.root #: Sequence
        results.concat(sequence.pluck(...))
      end

      results
    end

    #: () { (Document) -> void } -> void
    def apply!
      each do |document|
        yield document

        document.save!
      end
    end
  end
end
