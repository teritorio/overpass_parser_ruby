# OverpassParserRuby


## Installation

Install the gem and add to the application's Gemfile by executing:

```bash
bundle add overpass_parser_rust
```

If bundler is not being used to manage dependencies, install the gem by executing:

```bash
gem install overpass_parser_rust
```

## Usage

```ruby
OverpassParserRuby.parse("node(50.0, 8.0, 50.1, 8.1); out;").to_sql("postgres", "4326")
```

## Development

```bash
bundle exec rake compile
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/teritorio/overpass_parser_ruby.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
