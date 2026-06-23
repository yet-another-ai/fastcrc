# frozen_string_literal: true

require "test_helper"

class TestFastCRC < Minitest::Test
  SAMPLE_INPUTS = [
    "",
    "123456789",
    "hello world!",
    "\x00\xFF\x80".b
  ].freeze

  def test_that_it_has_a_version_number
    refute_nil ::FastCRC::VERSION
  end

  def test_crc32_checksum
    assert_equal 0xcbf43926, FastCRC::CRC32.checksum("123456789")
  end

  def test_crc32_hexdigest
    assert_equal "cbf43926", FastCRC::CRC32.hexdigest("123456789")
  end

  def test_crc64_nvme_checksum
    assert_equal 0xd9160d1fa8e418e3, FastCRC::CRC64NVMe.checksum("hello world!")
  end

  def test_crc64_nvme_hexdigest
    assert_equal "d9160d1fa8e418e3", FastCRC::CRC64NVMe.hexdigest("hello world!")
  end

  def test_empty_input
    assert_equal 0, FastCRC::CRC32.checksum("")
    assert_equal "00000000", FastCRC::CRC32.hexdigest("")
    assert_equal 0, FastCRC::CRC64NVMe.checksum("")
    assert_equal "0000000000000000", FastCRC::CRC64NVMe.hexdigest("")
  end

  def test_binary_input
    data = "\x00\xFF\x80".b

    assert_equal 0x81dda740, FastCRC::CRC32.checksum(data)
    assert_equal "81dda740", FastCRC::CRC32.hexdigest(data)
    assert_equal 0x8b09837e9f7e9d09, FastCRC::CRC64NVMe.checksum(data)
    assert_equal "8b09837e9f7e9d09", FastCRC::CRC64NVMe.hexdigest(data)
  end

  def test_non_string_input_raises
    assert_raises(TypeError) { FastCRC::CRC32.checksum(123) }
    assert_raises(TypeError) { FastCRC::CRC64NVMe.hexdigest(nil) }
  end

  def test_digest_crc_compatible_dialects
    FastCRCTestData::DIGEST_CRC_DIALECTS.each do |require_path, class_name|
      require "digest/#{require_path}"

      fastcrc_class = FastCRC.const_get(class_name)
      digest_crc_class = Digest.const_get(class_name)

      SAMPLE_INPUTS.each do |input|
        assert_equal digest_crc_class.checksum(input), fastcrc_class.checksum(input), "#{class_name}.checksum"
        assert_equal digest_crc_class.hexdigest(input), fastcrc_class.hexdigest(input), "#{class_name}.hexdigest"
      end
    end
  end

  def test_digest_crc_compatible_aliases
    assert_same FastCRC::CRC32MPEG, FastCRC::CRC32Mpeg
  end
end
