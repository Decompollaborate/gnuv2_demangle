# gnuv2_demangle

[![Build Status]][actions]
[![Latest Version]][crates.io]
![Crates.io MSRV]
[![Api Rustdoc]][rustdoc]

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

## Features

- Pure Rust implementation.
- No dependencies.
- Tries to mimic as closely as possible GNU's `c++filt`, including certain
  bugs. This behavior can be configured to avoid some inconsistencies and bugs
  from the original `c++filt` implementation.
- Compatible with `no_std` by default.
  - It still requires access to `alloc`.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
gnuv2_demangle = "0.1"
```

For a cli program, refer to [`g2dem`](https://github.com/Decompollaborate/gnuv2_demangle/tree/main/src/g2dem).

## Example

```rust
use gnuv2_demangle::{demangle, DemangleConfig};

let config = DemangleConfig::new();
let demangled = demangle("do_thing__C6StupidRC6StupidT1", &config);
assert_eq!(
    demangled.as_deref(),
    Ok("Stupid::do_thing(Stupid const &, Stupid const &) const"),
);
```

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
