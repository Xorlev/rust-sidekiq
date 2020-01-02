# Async Rust Sidekiq Client

[Sidekiq](https://github.com/mperham/sidekiq) client allowing to push jobs.
Using the [Sidekiq job
format](https://github.com/mperham/sidekiq/wiki/Job-Format) as reference.

## Dependencies

* [redis](https://github.com/mitsuhiko/redis-rs)
* [bb8](https://github.com/khuey/bb8)
* [serde_json](https://github.com/serde-rs/json)

## Installation

``` toml
[dependencies]
sidekiq = "0.8"
```

## Default environment variables

* REDIS_URL="redis://127.0.0.1/"

## REFERENCES

* <https://github.com/jkcclemens/paste>
* <http://julienblanchard.com/2015/using-resque-with-rust/>
* <https://github.com/d-unseductable/rust_sidekiq>
* <https://github.com/spk/maman>

## LICENSE

The MIT License

Copyright (c) 2016-2019 Laurent Arnoud <laurent@spkdev.net>

---
[![Build](https://img.shields.io/travis/spk/rust-sidekiq/master.svg)](https://travis-ci.org/spk/rust-sidekiq)
[![Version](https://img.shields.io/crates/v/sidekiq.svg)](https://crates.io/crates/sidekiq)
[![Documentation](https://img.shields.io/badge/doc-rustdoc-blue.svg)](https://docs.rs/sidekiq/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT "MIT")
[![Dependency status](https://deps.rs/repo/github/spk/rust-sidekiq/status.svg)](https://deps.rs/repo/github/spk/rust-sidekiq)
