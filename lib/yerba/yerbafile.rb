# frozen_string_literal: true

module Yerba
  class Yerbafile
    attr_reader :path

    def initialize(path)
      @path = File.expand_path(path)

      raise Yerba::Error, "Yerbafile not found: #{path}" unless File.exist?(@path)
    end

    def self.find(directory = Dir.pwd)
      path = locate(directory)

      path ? new(path) : nil
    end

    def self.find!(directory = Dir.pwd)
      find(directory) || raise(Yerba::Error, "No Yerbafile found")
    end

    def self.resolve(yerbafile = nil)
      case yerbafile
      when Yerbafile then yerbafile
      when String then new(yerbafile)
      when true, nil then find!
      end
    end

    def self.apply!(document, yerbafile = nil)
      resolve(yerbafile).apply(document)
    end

    def apply(document)
      document.apply_yerbafile(@path)

      document
    end

    def inspect
      "#<Yerba::Yerbafile path=#{@path.inspect}>"
    end
  end
end
