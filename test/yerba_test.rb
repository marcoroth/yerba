# frozen_string_literal: true

require "test_helper"

class YerbaTest < Minitest::Spec
  test "has a version number" do
    refute_nil Yerba::VERSION
  end
end
