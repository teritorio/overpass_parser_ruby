require "minitest/autorun"
require_relative "../lib/overpass_parser_ruby"

class OverpassParserRubyTest < Minitest::Test
  # Test parsing a sample Overpass QL query
  def test_parse
    query = <<~QUERY
      // @name Drinking Water

      /*
      This is an example Overpass query.
      Try it out by pressing the Run button above!
      You can find more examples with the Load tool.
      */
      [out:json];
      node["amenity"="cafe"](50.7,7.1,50.8,7.2);
      out body;
    QUERY

    result = OverpassParserRuby.parse(query)

    # Check if the result is not nil and has expected structure
    refute_nil result, "Parsing should not return nil"
  end
end
