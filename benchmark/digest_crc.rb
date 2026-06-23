#!/usr/bin/env ruby
# frozen_string_literal: true

require "benchmark"
require "bundler/setup"
require "fastcrc"
require "digest/crc32"
require "digest/crc64_nvme"

DEFAULT_ITERATIONS = 10
DEFAULT_SAMPLE_COUNT = 1_000
DEFAULT_BLOCK_SIZE = 8 * 1024

ITERATIONS = Integer(ENV.fetch("ITERATIONS", DEFAULT_ITERATIONS))
SAMPLE_COUNT = Integer(ENV.fetch("SAMPLE_COUNT", DEFAULT_SAMPLE_COUNT))
BLOCK_SIZE = Integer(ENV.fetch("BLOCK_SIZE", DEFAULT_BLOCK_SIZE))

SAMPLES = Array.new(SAMPLE_COUNT) do |sample_index|
  Random.new(sample_index).bytes(BLOCK_SIZE).b
end.freeze

BENCHMARKS = [
  {
    label: "CRC32",
    fastcrc_class: FastCRC::CRC32,
    digest_crc_class: Digest::CRC32
  },
  {
    label: "CRC64 NVMe",
    fastcrc_class: FastCRC::CRC64NVME,
    digest_crc_class: Digest::CRC64NVMe
  }
].freeze

def verify_matching_checksums!(fastcrc_class, digest_crc_class)
  SAMPLES.each do |sample|
    next if fastcrc_class.hexdigest(sample) == digest_crc_class.hexdigest(sample)

    raise "#{fastcrc_class} and #{digest_crc_class} produced different checksums"
  end
end

def update_all(digest)
  SAMPLES.each { |sample| digest.update(sample) }
  digest.hexdigest
end

puts "Ruby #{RUBY_VERSION} (#{RUBY_ENGINE})"
puts "digest-crc #{Gem.loaded_specs.fetch("digest-crc").version}"
puts "Samples: #{SAMPLE_COUNT} x #{BLOCK_SIZE} bytes"
puts "Iterations: #{ITERATIONS}"

BENCHMARKS.each do |benchmark|
  puts "\n#{benchmark.fetch(:label)}"

  fastcrc_class = benchmark.fetch(:fastcrc_class)
  digest_crc_class = benchmark.fetch(:digest_crc_class)

  verify_matching_checksums!(fastcrc_class, digest_crc_class)

  Benchmark.bm(28) do |runner|
    runner.report("#{fastcrc_class}#update") do
      ITERATIONS.times { update_all(fastcrc_class.new) }
    end

    runner.report("#{digest_crc_class}#update") do
      ITERATIONS.times { update_all(digest_crc_class.new) }
    end
  end
end
