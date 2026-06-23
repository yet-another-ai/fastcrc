# frozen_string_literal: true

require "bundler/gem_tasks"
require "minitest/test_task"

Minitest::TestTask.create

require "rubocop/rake_task"

RuboCop::RakeTask.new

require "rb_sys/extensiontask"

task build: :compile

GEMSPEC = Gem::Specification.load("fastcrc.gemspec")

RbSys::ExtensionTask.new("fastcrc", GEMSPEC) do |ext|
  ext.lib_dir = "lib/fastcrc"
end

namespace :benchmark do
  desc "Benchmark FastCRC against digest-crc"
  task digest_crc: :compile do
    ruby "benchmark/digest_crc.rb"
  end
end

task default: %i[compile test rubocop]
