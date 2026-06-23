#!/usr/bin/env ruby
# frozen_string_literal: true

require "benchmark"
require "bundler/setup"
require "fastcrc"

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
  ["CRC16", "crc16", "CRC16"],
  ["CRC16 CCITT", "crc16_ccitt", "CRC16CCITT"],
  ["CRC16 DNP", "crc16_dnp", "CRC16DNP"],
  ["CRC16 Genibus", "crc16_genibus", "CRC16Genibus"],
  ["CRC16 Kermit", "crc16_kermit", "CRC16Kermit"],
  ["CRC16 Modbus", "crc16_modbus", "CRC16Modbus"],
  ["CRC16 QT", "crc16_qt", "CRC16QT"],
  ["CRC16 USB", "crc16_usb", "CRC16USB"],
  ["CRC16 X25", "crc16_x_25", "CRC16X25"],
  ["CRC16 XModem", "crc16_xmodem", "CRC16XModem"],
  ["CRC16 ZModem", "crc16_zmodem", "CRC16ZModem"],
  ["CRC32", "crc32", "CRC32"],
  ["CRC32 BZip2", "crc32_bzip2", "CRC32BZip2"],
  ["CRC32c", "crc32c", "CRC32c"],
  ["CRC32 Jam", "crc32_jam", "CRC32Jam"],
  ["CRC32 MPEG", "crc32_mpeg", "CRC32MPEG"],
  ["CRC32 POSIX", "crc32_posix", "CRC32POSIX"],
  ["CRC32 XFER", "crc32_xfer", "CRC32XFER"],
  ["CRC64", "crc64", "CRC64"],
  ["CRC64 Jones", "crc64_jones", "CRC64Jones"],
  ["CRC64 NVMe", "crc64_nvme", "CRC64NVMe"],
  ["CRC64 XZ", "crc64_xz", "CRC64XZ"]
].map do |label, require_path, class_name|
  require "digest/#{require_path}"

  {
    label: label,
    fastcrc_class: FastCRC.const_get(class_name),
    digest_crc_class: Digest.const_get(class_name)
  }
end.freeze

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
