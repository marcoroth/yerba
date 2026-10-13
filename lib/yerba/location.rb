# frozen_string_literal: true

module Yerba
  Location = Data.define(
    :start_line, #: Integer
    :start_column, #: Integer
    :end_line, #: Integer
    :end_column, #: Integer
    :start_offset, #: Integer
    :end_offset #: Integer
  )
end
