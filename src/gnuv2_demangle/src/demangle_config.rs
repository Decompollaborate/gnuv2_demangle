/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct DemangleConfig {
    /// Recreate a c++filt bug where it won't emit the
    /// "global constructors keyed to " prefix for a namespaced function.
    pub preserve_namespaced_global_constructor_bug: bool,
    /// By default g++ subtracts 1 from the length of array arguments, thus
    /// producing a confusing mangled name.
    ///
    /// c++filt uses this length as-is, which produces a demangled symbol that
    /// does not match the original C++ symbol.
    ///
    /// This setting adds 1 to the length, making the demangled symbol match
    /// more accurately the real symbol.
    pub fix_array_length_arg: bool,

    /// Recognize and demangle symbols prefixed by `_GLOBAL_$F$`.
    ///
    /// c++filt does not recognizes this prefix, so it tries to demangle it as
    /// other mangled kinds, like functions, methods, etc.
    ///
    /// When turned on, the symbol gets demangled the same way `_GLOBAL_$I$`
    /// and `_GLOBAL_$D$` are demangled, but the word "frames" is used instead
    /// of "constructors" or "destructors". This name is made-up based on some
    /// usages from projects that have this symbol present.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.demangle_global_keyed_frames = false;
    ///
    /// let demangled = demangle("_GLOBAL_$F$__7istreamiP9streambufP7ostream", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("istream::_GLOBAL_$F$(int, streambuf *, ostream *)")
    /// );
    /// let demangled = demangle("_GLOBAL_$F$__default_terminate", &config);
    /// assert!(
    ///     demangled.is_err()
    /// );
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.demangle_global_keyed_frames = true;
    ///
    /// let demangled = demangle("_GLOBAL_$F$__7istreamiP9streambufP7ostream", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("global frames keyed to istream::istream(int, streambuf *, ostream *)")
    /// );
    /// let demangled = demangle("_GLOBAL_$F$__default_terminate", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("global frames keyed to __default_terminate")
    /// );
    /// ```
    pub demangle_global_keyed_frames: bool,
}

impl DemangleConfig {
    /// The default config mimics the default (rather questionable) c++filt's
    /// behavior, including what may be considered c++filt bugs.
    pub fn new() -> Self {
        Self {
            preserve_namespaced_global_constructor_bug: true,
            fix_array_length_arg: false,
            demangle_global_keyed_frames: false,
        }
    }

    /// Avoid using any option that mimics c++filt faults.
    pub fn new_no_cfilt_mimics() -> Self {
        Self {
            preserve_namespaced_global_constructor_bug: false,
            fix_array_length_arg: true,
            demangle_global_keyed_frames: true,
        }
    }
}

impl Default for DemangleConfig {
    fn default() -> Self {
        Self::new()
    }
}
