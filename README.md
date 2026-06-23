# FastCRC

Fast CRC checksum computation for Ruby, backed by Rust SIMD-accelerated implementations.

Supported algorithms:

- `FastCRC::CRC32` — CRC-32/IEEE via [`crc-fast`](https://crates.io/crates/crc-fast)
- `FastCRC::CRC64NVME` — CRC-64/NVME via [`crc-fast`](https://crates.io/crates/crc-fast)

## Installation

Install the gem and add to the application's Gemfile:

```bash
bundle add fastcrc
```

Or install directly:

```bash
gem install fastcrc
```

## Usage

```ruby
require "fastcrc"

FastCRC::CRC32.hexdigest("123456789")
# => "cbf43926"

FastCRC::CRC32.checksum("123456789")
# => 3421780262

FastCRC::CRC64NVME.hexdigest("hello world!")
# => "d9160d1fa8e418e3"

FastCRC::CRC64NVME.checksum("hello world!")
# => 15655158020120117219
```

Both APIs accept binary strings:

```ruby
FastCRC::CRC32.hexdigest("\x00\xFF\x80".b)
```

### Incremental updates

For large files or streaming input, create an instance and call `#update` with each chunk:

```ruby
digest = FastCRC::CRC32.new

File.open("large.bin", "rb") do |file|
  while (chunk = file.read(8192))
    digest.update(chunk)
  end
end

digest.hexdigest
```

With a fiber scheduler (for example via the [`async`](https://rubygems.org/gems/async) gem), chunked reads can yield between I/O and computation:

```ruby
require "async"

Async do |task|
  digest = FastCRC::CRC32.new

  File.open("large.bin", "rb") do |file|
    while (chunk = file.read(8192))
      digest.update(chunk)
      task.yield
    end
  end

  puts digest.hexdigest
end
```

Instance methods mirror the class methods:

- `#update(data)` — append data to the running checksum
- `#checksum` — current checksum as an integer
- `#hexdigest` — current checksum as a lowercase hex string
- `#reset` — reset internal state

## Development

After checking out the repo, run `bin/setup` to install dependencies. Then, run `rake test` to run the tests. You can also run `bin/console` for an interactive prompt.

To install this gem onto your local machine, run `bundle exec rake install`.

### Benchmarks

Benchmark FastCRC against the [`digest-crc`](https://github.com/postmodern/digest-crc) gem with:

```bash
bundle exec rake benchmark:digest_crc
```

The benchmark compares incremental `#update` performance for the overlapping `CRC32` and `CRC64 NVMe` algorithms. You can tune the workload with `ITERATIONS`, `SAMPLE_COUNT`, and `BLOCK_SIZE` environment variables:

```bash
ITERATIONS=5 SAMPLE_COUNT=500 BLOCK_SIZE=16384 bundle exec rake benchmark:digest_crc
```

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
