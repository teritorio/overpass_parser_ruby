# frozen_string_literal: true

require "bundler/gem_tasks"
require "rubocop/rake_task"

RuboCop::RakeTask.new

require "rb_sys/extensiontask"

task build: :compile

GEMSPEC = Gem::Specification.load("overpass_parser_ruby.gemspec")

RbSys::ExtensionTask.new("overpass_parser_ruby", GEMSPEC) do |ext|
  ext.lib_dir = "lib/overpass_parser_ruby"
end

task default: %i[compile rubocop]
