# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for all overloadable operators.
- Add support for some mangling variants used by ProDG.

## [0.2.0] - 2025-09-28

### Added

- First public version of `g2dem-web`.
  - Visit it at <https://decompollaborate.github.io/gnuv2_demangle/>
- `DemangleConfig::ellipsis_emit_space_after_comma`: If set then emit an space
  between the ellipsis and the last comma in the argument list.
- Support for method pointers as arguments.
- Support for 128bits integers.
- Operator `delete []`.
- Support `enum`s as templated values.
- Support for reuse of templated values (`Y`).
- `DemangleConfig::fix_array_in_return_position`: Emit proper syntax for
  templated functions returning pointers to arrays.
- `g2dem`: `--mode`/`-m` flag: Select between `c++filt` and `g2dem` demangling
  modes.

### Changed

- General code cleanups.
- Renames:
  - `DemangleConfig::new_mimic_cfilt` -> `DemangleConfig::new_cfilt`.
  - `DemangleConfig::new_no_cfilt_mimics` -> `DemangleConfig::new_g2dem`.
- Change `DemangleConfig::preserve_namespaced_global_constructor_bug` to
  `DemangleConfig::fix_namespaced_global_constructor_bug`.
  - This means the opposite to what it previously meant, so it can be
    consistent with the other options.
- `DemangleConfig::new` now uses `DemangleConfig::new_g2dem` instead of
  `DemangleConfig::new_cfilt`.

### Fixed

- Fix emitting a comma when the only argument in a function is an ellipsis.
- Fix function pointers inside function pointers, I think.
- Fix sometimes being unable to demangle templated functions.
- Fix emitting empty parenthesis on non-pointer arrays types.
- Fix incorrect syntax used for arrays as return types in function pointers.
- Fix failing to demangle templated functions with large integer values.
- Fix demangling templated functions inside classes and namespaces.
- Fix array sizes in templated functions.

## [0.1.0] - 2025-09-18

- Initial release.

[unreleased]: https://github.com/Decompollaborate/gnuv2_demangle/compare/0.2.0...main
[0.2.0]: https://github.com/Decompollaborate/gnuv2_demangle/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/Decompollaborate/gnuv2_demangle/releases/tag/0.1.0
