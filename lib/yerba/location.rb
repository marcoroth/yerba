# frozen_string_literal: true

module Yerba
  Location = Data.define(
    :start_line, :start_column, :end_line, :end_column, :start_offset, :end_offset,
    :start_character_column, :end_character_column
  ) do
    def initialize(start_column:, end_column:, start_character_column: start_column, end_character_column: end_column, **)
      super
    end
  end
end
