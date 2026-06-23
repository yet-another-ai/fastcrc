# frozen_string_literal: true

$LOAD_PATH.unshift File.expand_path("../lib", __dir__)
require "fastcrc"

module FastCRCTestData
  DIGEST_CRC_DIALECTS = [
    %w[crc16 CRC16],
    %w[crc16_ccitt CRC16CCITT],
    %w[crc16_dnp CRC16DNP],
    %w[crc16_genibus CRC16Genibus],
    %w[crc16_kermit CRC16Kermit],
    %w[crc16_modbus CRC16Modbus],
    %w[crc16_qt CRC16QT],
    %w[crc16_usb CRC16USB],
    %w[crc16_x_25 CRC16X25],
    %w[crc16_xmodem CRC16XModem],
    %w[crc16_zmodem CRC16ZModem],
    %w[crc32 CRC32],
    %w[crc32_bzip2 CRC32BZip2],
    %w[crc32c CRC32c],
    %w[crc32_jam CRC32Jam],
    %w[crc32_mpeg CRC32MPEG],
    %w[crc32_posix CRC32POSIX],
    %w[crc32_xfer CRC32XFER],
    %w[crc64 CRC64],
    %w[crc64_jones CRC64Jones],
    %w[crc64_nvme CRC64NVMe],
    %w[crc64_xz CRC64XZ]
  ].freeze

  FASTCRC_CLASS_NAMES = DIGEST_CRC_DIALECTS.map { |_require_path, class_name| class_name.to_sym }.freeze
end

require "minitest/autorun"
