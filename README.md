# FastCRC

Fast CRC checksum computation for Ruby, backed by Rust SIMD-accelerated implementations.

Supported algorithms:

- `FastCRC::CRC32` — CRC-32/IEEE via [`crc32fast`](https://crates.io/crates/crc32fast)
- `FastCRC::CRC64NVME` — CRC-64/NVME via [`crc64fast-nvme`](https://crates.io/crates/crc64fast-nvme)

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

## Development

After checking out the repo, run `bin/setup` to install dependencies. Then, run `rake test` to run the tests. You can also run `bin/console` for an interactive prompt.

To install this gem onto your local machine, run `bundle exec rake install`.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
