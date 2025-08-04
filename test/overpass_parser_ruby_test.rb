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

    sql = result.to_sql("postgres", 4326, proc { |s| "_#{s}_" })

    refute_nil sql, "SQL generation should not return nil"
  end

  def test_exception
    query = <<~QUERY
      foo bar !
    QUERY

    result = OverpassParserRuby.parse(query)
    refute_nil result, "Parsing should not pass"
  rescue OverpassParserRuby::ParsingError => e
    # Check if the result is not nil and has expected structure
  end

  def test_selectors_one
    tree = OverpassParserRuby.parse("node[shop];")
    selectors = tree.all_selectors.first

    assert_equal(
      selectors.matches({ "shop" => "supermarket" }),
      ["shop"]
    )

    assert_equal(selectors.keys, ["shop"])
    assert_equal(selectors.to_sql("postgres", 4326, nil), "tags?'shop'")
    assert_equal(selectors.to_overpass, "[shop]")
  end

  def test_selectors_all
    tree = OverpassParserRuby.parse("[out:json][timeout:25];
area(id:1,2,3)->.a;
.a out center meta;
(
  node[a=b];
  way[d];
);
out center meta;
")
    selectors = tree.all_selectors.collect(&:to_overpass)

    assert_equal(selectors, ["[a=b]", "[d]"])
  end
end
