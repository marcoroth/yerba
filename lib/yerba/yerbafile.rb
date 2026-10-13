# frozen_string_literal: true

module Yerba
  class Yerbafile
    # @rbs!
    #   # Defined in the C extension.
    #   def self.locate: (?String? directory) -> String?

    attr_reader :path #: String

    #: (String path) -> void
    def initialize(path)
      @path = File.expand_path(path)

      raise Yerba::Error, "Yerbafile not found: #{path}" unless File.exist?(@path)
    end

    #: (?String directory) -> Yerbafile?
    def self.find(directory = Dir.pwd)
      path = locate(directory)

      path ? new(path) : nil
    end

    #: (?String directory) -> Yerbafile
    def self.find!(directory = Dir.pwd)
      find(directory) || raise(Yerba::Error, "No Yerbafile found")
    end

    #: (?(Yerbafile | String | true)? yerbafile) -> Yerbafile
    def self.resolve(yerbafile = nil)
      case yerbafile
      when Yerbafile then yerbafile
      when String then new(yerbafile)
      when true, nil then find!
      end
    end

    #: (Document document, ?(Yerbafile | String | true)? yerbafile) -> Document
    def self.apply!(document, yerbafile = nil)
      resolve(yerbafile).apply(document)
    end

    #: (Document document) -> Document
    def apply(document)
      document.apply_yerbafile(@path)

      document
    end

    #: () -> String
    def inspect
      "#<Yerba::Yerbafile path=#{@path.inspect}>"
    end
  end
end
