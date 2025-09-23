# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `DemangleConfig::ellipsis_emit_space_after_comma`: If set then emit an space
  between the ellipsis and the last comma in the argument list.
  - Defaults to `false` to mimic c++filt behavior.
- Support for method pointers as arguments.
- Support for 128bits integers.
- Operator `delete []`.
- Support `enum`s as templated values.

### Changed

- General code cleanups.

### Fixed

- Fix emitting a comma when the only argument in a function is an ellipsis.
- Fix function pointers inside function pointers, I think.
- Fix sometimes being unable to demangle templated functions.

## [0.1.0] - 2025-09-18

- Initial release.

[unreleased]: https://github.com/Decompollaborate/gnuv2_demangle/compare/0.1.0...main
[0.1.0]: https://github.com/Decompollaborate/gnuv2_demangle/releases/tag/0.1.0
