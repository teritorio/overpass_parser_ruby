# frozen_string_literal: true

require_relative "overpass_parser_ruby/version"

begin
  require "overpass_parser_ruby/overpass_parser_ruby"
rescue LoadError
  # Fallback for development
  require File.expand_path("../target/debug/liboverpass_parser_ruby", __dir__)
end
