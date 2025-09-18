# gnuv2_demangle

[![Build Status]][actions] [![Latest Version]][crates.io] ![Crates.io MSRV] [![Api Rustdoc]][rustdoc]

[Build Status]: https://img.shields.io/github/actions/workflow/status/Decompollaborate/gnuv2_demangle/build_and_publish_crate.yml
[actions]: https://github.com/Decompollaborate/gnuv2_demangle/actions
[Latest Version]: https://img.shields.io/crates/v/gnuv2_demangle
[crates.io]: https://crates.io/crates/gnuv2_demangle
[Crates.io MSRV]: https://img.shields.io/crates/msrv/gnuv2_demangle
[Api Rustdoc]: https://img.shields.io/badge/api-rustdoc-blue
[rustdoc]: https://docs.rs/gnuv2_demangle

A GNU V2 C++ symbol demangler.

## Important note

This crate demangles symbols for the outdated and no-longer-used V2 ABI
mangling scheme of GNU g++. It is very unlikely this is actually the you are
looking for, since this stuff is ancient.

Only use this crate if you are sure you want to use the g++ mangling scheme
used in gcc 2.9 and older.

It is more likely you are looking for crates like
[`cpp_demangle`](https://crates.io/crates/cpp_demangle),
[`symbolic-demangle`](https://crates.io/crates/symbolic-demangle)
or [`cplus_demangle`](https://crates.io/crates/cplus_demangle)

## What's in here?

This repository is the home of two Rust crates:

- [`gnuv2_demangle`](src/gnuv2_demangle/): The demangler library crate for GNU
  V2 C++ mangled symbols.
- [`g2dem`](src/g2dem/): A `c++filt` clone that uses `gnuv2_demangle` to
  demangle symbols.

Please refer to their respective READMEs for more information about each one.

## Implementation notes

I implemented this crate by throwing a lot of symbols at an old version of
`c++filt` (2.9 ish), looking the output and trying to make sense of the
demangling process.

Because of this, you can expect some inconsistencies, mismangled symbols and
other issues while using this crate. If you find any problem feel free to reach
out via Github issues or a PR.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

## Versioning and changelog

This library follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
We try to always keep backwards compatibility, so no breaking changes should
happen until a major release (i.e. jumping from 1.X.X to 2.0.0).

To see what changed on each release visit either the
[CHANGELOG.md](https://github.com/Decompollaborate/gnuv2_demangle/blob/main/CHANGELOG.md)
file or check the [releases page on Github](https://github.com/Decompollaborate/gnuv2_demangle/releases).
You can also use [this link](https://github.com/Decompollaborate/gnuv2_demangle/releases/latest)
to check the latest release.
