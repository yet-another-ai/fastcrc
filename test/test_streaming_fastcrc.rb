# frozen_string_literal: true

require "test_helper"
require "tempfile"
require "async"

class TestStreamingFastCRC < Minitest::Test
  CHUNK_SIZE = 4096
  CRC_CLASSES = FastCRCTestData::FASTCRC_CLASS_NAMES.map { |class_name| FastCRC.const_get(class_name) }.freeze

  def test_crc32_incremental_matches_one_shot
    data = ("123456789" * 1000) + "\x00\xFF\x80".b
    digest = FastCRC::CRC32.new

    update_in_chunks(digest, data)

    assert_equal FastCRC::CRC32.checksum(data), digest.checksum
    assert_equal FastCRC::CRC32.hexdigest(data), digest.hexdigest
  end

  def test_crc64_nvme_incremental_matches_one_shot
    data = ("hello world!" * 500) + "\x00\xFF\x80".b
    digest = FastCRC::CRC64NVMe.new

    update_in_chunks(digest, data)

    assert_equal FastCRC::CRC64NVMe.checksum(data), digest.checksum
    assert_equal FastCRC::CRC64NVMe.hexdigest(data), digest.hexdigest
  end

  def test_incremental_matches_one_shot_for_supported_dialects
    data = ("123456789" * 1000) + "\x00\xFF\x80".b

    CRC_CLASSES.each do |crc_class|
      digest = crc_class.new
      update_in_chunks(digest, data)

      assert_equal crc_class.checksum(data), digest.checksum, "#{crc_class}#checksum"
      assert_equal crc_class.hexdigest(data), digest.hexdigest, "#{crc_class}#hexdigest"
    end
  end

  def test_reset_clears_state
    digest = FastCRC::CRC32.new
    digest.update("partial")
    digest.reset
    digest.update("123456789")

    assert_equal FastCRC::CRC32.hexdigest("123456789"), digest.hexdigest
  end

  def test_crc32_file_streaming_with_fiber_scheduler
    data = ("123456789" * 10_000) + "\x00\xFF\x80".b
    expected = FastCRC::CRC32.hexdigest(data)

    actual = digest_file_with_fiber_scheduler(FastCRC::CRC32, data)

    assert_equal expected, actual
  end

  def test_crc64_nvme_file_streaming_with_fiber_scheduler
    data = ("hello world!" * 10_000) + "\x00\xFF\x80".b
    expected = FastCRC::CRC64NVMe.hexdigest(data)

    actual = digest_file_with_fiber_scheduler(FastCRC::CRC64NVMe, data)

    assert_equal expected, actual
  end

  def test_fiber_scheduler_interleaved_updates
    data = "fiber-scheduler-test" * 5000
    expected = FastCRC::CRC32.hexdigest(data)

    actual = digest_with_fiber_scheduler(FastCRC::CRC32.new, data)

    assert_equal expected, actual
  end

  private

  def update_in_chunks(digest, data)
    data.bytes.each_slice(CHUNK_SIZE) do |slice|
      digest.update(slice.pack("C*"))
    end
  end

  def digest_with_fiber_scheduler(digest, data)
    Async do |task|
      data.bytes.each_slice(CHUNK_SIZE) do |slice|
        digest.update(slice.pack("C*"))
        task.yield
      end
      digest.hexdigest
    end.wait
  end

  def digest_file_with_fiber_scheduler(digest_class, data)
    file = Tempfile.new("fastcrc")
    file.write(data)
    file.flush

    digest_file_path_with_fiber_scheduler(digest_class, file.path)
  ensure
    file&.close
    file&.unlink
  end

  def digest_file_path_with_fiber_scheduler(digest_class, path)
    Async do |task|
      digest = digest_class.new

      File.open(path, "rb") do |io|
        while (chunk = io.read(CHUNK_SIZE))
          digest.update(chunk)
          task.yield
        end
      end

      digest.hexdigest
    end.wait
  end
end
