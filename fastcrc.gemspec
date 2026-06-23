# frozen_string_literal: true

require_relative "lib/fastcrc/version"

Gem::Specification.new do |spec|
  spec.name = "fastcrc"
  spec.version = FastCRC::VERSION
  spec.authors = ["Yet Another AI"]
  spec.email = ["rubygems@yetanother.ai"]

  spec.summary = "Fast CRC checksum computation for Ruby, backed by Rust SIMD-accelerated implementations."
  spec.description = "Fast CRC checksum for Ruby using crc-fast " \
                      "via Magnus/Rust bindings."
  spec.homepage = "https://github.com/dsh0416/yet-another-ai/fastcrc"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.2.0"
  spec.required_rubygems_version = ">= 3.3.11"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  gemspec = File.basename(__FILE__)
  spec.files = IO.popen(%w[git ls-files -z], chdir: __dir__, err: IO::NULL) do |ls|
    ls.readlines("\x0", chomp: true).reject do |f|
      (f == gemspec) ||
        f.start_with?(*%w[bin/ Gemfile .gitignore test/ .github/ .rubocop.yml])
    end
  end
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/fastcrc/extconf.rb"]

  spec.add_dependency "rb_sys", "~> 0.9.91"
end
