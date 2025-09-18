# g2dem

A `c++filt` clone cli tool for GNU V2 mangled symbols.

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

## Installation

Prebuilt binaries are available on the [Github releases](https://github.com/Decompollaborate/gnuv2_demangle/releases/latest)
page.

Alternatively, you can install the latest version with Cargo. Note you'll need
[an updated Rust toolchain](https://www.rust-lang.org/tools/install) to
install via Cargo.

```bash
cargo install g2dem --locked
```

## Usage

Pass any number of symbols as arguments to `g2dem` to demangle them. If a name
can't be demangled then it will be echoed to `stdout` as-is.

```bash
$ g2dem do_thing__C6StupidRC6StupidT1 not_a_mangled_sym
Stupid::do_thing(Stupid const &, Stupid const &) const
not_a_mangled_sym
```

If no symbols are passed on the command line, then `g2dem` will read from
`stdin` instead.

```bash
$ echo do_thing__C6StupidRC6StupidT1 | g2dem
Stupid::do_thing(Stupid const &, Stupid const &) const
```

Pass `--help` to see other available options.

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
