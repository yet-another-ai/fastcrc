# FastCRC

FastCRC is a Ruby gem for fast CRC checksum computation, backed by a Rust native extension and the [`crc-fast`](https://crates.io/crates/crc-fast) implementation.

It is built for Ruby applications that need CRC checksums in hot paths: object storage clients, upload/download verification, archival formats, wire protocols, and any workload where a pure Ruby or straightforward C implementation becomes visible in profiles.

## Why FastCRC?

Ruby 3 made Rust a normal part of the Ruby ecosystem. YJIT is implemented in Rust, Ruby projects increasingly keep a Rust toolchain nearby, and native Rust extensions are much easier to adopt than they used to be.

At the same time, checksum requirements are getting more specific. APIs such as AWS S3's newer checksum flows require CRC64NVMe support, and that algorithm benefits from a backend designed for SIMD-friendly throughput.

FastCRC exists to make those optimized CRC implementations available from Ruby with a small, familiar API:

- Rust-backed CRC implementations through `crc-fast` and [`magnus`](https://github.com/matsadler/magnus).
- CRC-16, CRC-32, and CRC-64 dialects that overlap with [`digest-crc`](https://github.com/postmodern/digest-crc).
- CRC64NVMe support for modern storage APIs.
- One-shot and incremental APIs for strings, files, and streaming input.
- Fiber Scheduler and [`async`](https://rubygems.org/gems/async)-friendly chunked processing for better I/O behavior.

On supported workloads, the Rust backend can be tens of times faster than naive CRC implementations, commonly in the 60x to 100x range.

## Installation

Add the gem to your application:

```bash
bundle add fastcrc
```

Or install it directly:

```bash
gem install fastcrc
```

FastCRC requires Ruby 3.2 or newer. Installing from source requires a working Rust toolchain because the gem builds a native extension.

## Usage

Require the gem and call the CRC class you need:

```ruby
require "fastcrc"

FastCRC::CRC32.hexdigest("123456789")
# => "cbf43926"

FastCRC::CRC32.checksum("123456789")
# => 3421780262

FastCRC::CRC64NVMe.hexdigest("hello world!")
# => "d9160d1fa8e418e3"

FastCRC::CRC64NVMe.checksum("hello world!")
# => 15655158020120117219
```

The APIs accept binary strings:

```ruby
FastCRC::CRC32.hexdigest("\x00\xFF\x80".b)
# => "81dda740"
```

## Streaming Input

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

Instance methods mirror the one-shot class methods:

- `#update(data)` appends data to the running checksum.
- `#checksum` returns the current checksum as an integer.
- `#hexdigest` returns the current checksum as a lowercase hex string.
- `#reset` clears the current state so the instance can be reused.

### Fiber Scheduler And Async

FastCRC's incremental API works well with scheduler-aware I/O. With a fiber scheduler, your code can read a chunk, update the checksum, and yield so other fibers can continue making progress:

```ruby
require "async"

Async do |task|
  digest = FastCRC::CRC64NVMe.new

  File.open("large.bin", "rb") do |file|
    while (chunk = file.read(8192))
      digest.update(chunk)
      task.yield
    end
  end

  puts digest.hexdigest
end
```

This is useful for clients that validate large uploads or downloads while sharing an event loop with other network or file I/O.

## Supported Algorithms

FastCRC mirrors the CRC dialects that overlap with `digest-crc`.

CRC-16:

- `FastCRC::CRC16`
- `FastCRC::CRC16CCITT`
- `FastCRC::CRC16DNP`
- `FastCRC::CRC16Genibus`
- `FastCRC::CRC16Kermit`
- `FastCRC::CRC16Modbus`
- `FastCRC::CRC16QT`
- `FastCRC::CRC16USB`
- `FastCRC::CRC16X25`
- `FastCRC::CRC16XModem`
- `FastCRC::CRC16ZModem`

CRC-32:

- `FastCRC::CRC32`
- `FastCRC::CRC32BZip2`
- `FastCRC::CRC32c`
- `FastCRC::CRC32Jam`
- `FastCRC::CRC32MPEG`
- `FastCRC::CRC32Mpeg`
- `FastCRC::CRC32POSIX`
- `FastCRC::CRC32XFER`

CRC-64:

- `FastCRC::CRC64`
- `FastCRC::CRC64Jones`
- `FastCRC::CRC64NVMe`
- `FastCRC::CRC64XZ`

## Benchmarks

Benchmark FastCRC against `digest-crc` with:

```bash
bundle exec rake benchmark:digest_crc
```

The benchmark compares incremental `#update` performance for the supported `digest-crc` overlap. Tune the workload with `ITERATIONS`, `SAMPLE_COUNT`, and `BLOCK_SIZE`:

```bash
ITERATIONS=5 SAMPLE_COUNT=500 BLOCK_SIZE=16384 bundle exec rake benchmark:digest_crc
```

Benchmark results on M4 Pro CPU:

```
Ruby 4.0.5 (ruby)
digest-crc 0.7.0
Samples: 1000 x 8192 bytes
Iterations: 10

CRC16
                                   user     system      total        real
FastCRC::CRC16#update          0.001703   0.000031   0.001734 (  0.001733)
Digest::CRC16#update           0.145404   0.000458   0.145862 (  0.146030)

CRC16 CCITT
                                   user     system      total        real
FastCRC::CRC16CCITT#update     0.001919   0.000018   0.001937 (  0.001937)
Digest::CRC16CCITT#update      0.161412   0.000472   0.161884 (  0.162138)

CRC16 DNP
                                   user     system      total        real
FastCRC::CRC16DNP#update       0.146592   0.000701   0.147293 (  0.147627)
Digest::CRC16DNP#update        0.139462   0.000579   0.140041 (  0.140277)

CRC16 Genibus
                                   user     system      total        real
FastCRC::CRC16Genibus#update   0.001894   0.000002   0.001896 (  0.001894)
Digest::CRC16Genibus#update    0.156035   0.000393   0.156428 (  0.156633)

CRC16 Kermit
                                   user     system      total        real
FastCRC::CRC16Kermit#update    0.001428   0.000003   0.001431 (  0.001430)
Digest::CRC16Kermit#update     0.140393   0.000410   0.140803 (  0.140935)

CRC16 Modbus
                                   user     system      total        real
FastCRC::CRC16Modbus#update    0.001444   0.000004   0.001448 (  0.001449)
Digest::CRC16Modbus#update     0.140009   0.000583   0.140592 (  0.140810)

CRC16 QT
                                   user     system      total        real
FastCRC::CRC16QT#update        0.001421   0.000004   0.001425 (  0.001424)
Digest::CRC16QT#update         0.139679   0.000327   0.140006 (  0.140081)

CRC16 USB
                                   user     system      total        real
FastCRC::CRC16USB#update       0.001429   0.000036   0.001465 (  0.001465)
Digest::CRC16USB#update        0.140984   0.000484   0.141468 (  0.141715)

CRC16 X25
                                   user     system      total        real
FastCRC::CRC16X25#update       0.001426   0.000006   0.001432 (  0.001433)
Digest::CRC16X25#update        0.139800   0.000577   0.140377 (  0.140534)

CRC16 XModem
                                   user     system      total        real
FastCRC::CRC16XModem#update    0.001892   0.000001   0.001893 (  0.001892)
Digest::CRC16XModem#update     0.156624   0.000414   0.157038 (  0.157100)

CRC16 ZModem
                                   user     system      total        real
FastCRC::CRC16ZModem#update    0.001958   0.000001   0.001959 (  0.001961)
Digest::CRC16ZModem#update     0.157277   0.000434   0.157711 (  0.157864)

CRC32
                                   user     system      total        real
FastCRC::CRC32#update          0.001688   0.000003   0.001691 (  0.001691)
Digest::CRC32#update           0.138896   0.000670   0.139566 (  0.139735)

CRC32 BZip2
                                   user     system      total        real
FastCRC::CRC32BZip2#update     0.001880   0.000009   0.001889 (  0.001889)
Digest::CRC32BZip2#update      0.137953   0.000616   0.138569 (  0.138768)

CRC32c
                                   user     system      total        real
FastCRC::CRC32c#update         0.001811   0.000001   0.001812 (  0.001813)
Digest::CRC32c#update          0.138962   0.000351   0.139313 (  0.139526)

CRC32 Jam
                                   user     system      total        real
FastCRC::CRC32Jam#update       0.001480   0.000003   0.001483 (  0.001481)
Digest::CRC32Jam#update        0.138885   0.000232   0.139117 (  0.139282)

CRC32 MPEG
                                   user     system      total        real
FastCRC::CRC32MPEG#update      0.001864   0.000001   0.001865 (  0.001864)
Digest::CRC32MPEG#update       0.137355   0.000277   0.137632 (  0.137757)

CRC32 POSIX
                                   user     system      total        real
FastCRC::CRC32POSIX#update     0.002034   0.000009   0.002043 (  0.002049)
Digest::CRC32POSIX#update      0.140943   0.000757   0.141700 (  0.142165)

CRC32 XFER
                                   user     system      total        real
FastCRC::CRC32XFER#update      0.002024   0.000004   0.002028 (  0.002029)
Digest::CRC32XFER#update       0.144710   0.001249   0.145959 (  0.146233)

CRC64
                                   user     system      total        real
FastCRC::CRC64#update          0.001410   0.000001   0.001411 (  0.001408)
Digest::CRC64#update           0.141028   0.000632   0.141660 (  0.141901)

CRC64 Jones
                                   user     system      total        real
FastCRC::CRC64Jones#update     0.001432   0.000001   0.001433 (  0.001433)
Digest::CRC64Jones#update      0.145503   0.001425   0.146928 (  0.147631)

CRC64 NVMe
                                   user     system      total        real
FastCRC::CRC64NVMe#update      0.001486   0.000001   0.001487 (  0.001486)
Digest::CRC64NVMe#update       0.143874   0.000934   0.144808 (  0.145643)

CRC64 XZ
                                   user     system      total        real
FastCRC::CRC64XZ#update        0.001425   0.000002   0.001427 (  0.001424)
Digest::CRC64XZ#update         0.139581   0.000869   0.140450 (  0.140569)
```

## Development

After checking out the repo, install dependencies and run the test suite:

```bash
bin/setup
rake test
```

Run the full default task with:

```bash
rake
```

To install this gem onto your local machine:

```bash
bundle exec rake install
```

## License

FastCRC is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
