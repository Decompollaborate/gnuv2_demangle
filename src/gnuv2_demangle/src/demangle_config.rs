/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/// Tweak how a symbol should be disassembled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct DemangleConfig {
    /// Recreate a c++filt bug where it won't emit the
    /// "global constructors keyed to " prefix for a namespaced function.
    ///
    /// This is just another c++filt compatibility setting.
    ///
    /// # Examples
    ///
    /// Turning on this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.preserve_namespaced_global_constructor_bug = true;
    ///
    /// let demangled = demangle("_GLOBAL_$I$__Q210Scenegraph10Scenegraph", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("Scenegraph::Scenegraph::Scenegraph(void)")
    /// );
    /// ```
    ///
    /// The setting turned off:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.preserve_namespaced_global_constructor_bug = false;
    ///
    /// let demangled = demangle("_GLOBAL_$I$__Q210Scenegraph10Scenegraph", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("global constructors keyed to Scenegraph::Scenegraph::Scenegraph(void)")
    /// );
    pub preserve_namespaced_global_constructor_bug: bool,

    /// By default g++ subtracts 1 from the length of array arguments, thus
    /// producing a confusing mangled name.
    ///
    /// c++filt uses this length as-is, which produces a demangled symbol that
    /// does not match the original C++ symbol.
    ///
    /// This setting adds 1 to the length, making the demangled symbol match
    /// more accurately the real symbol.
    ///
    /// This is just another c++filt compatibility setting.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_length_arg = false;
    ///
    /// let demangled = demangle("simpler_array__FPA41_A24_Ci", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("simpler_array(int const (*)[41][24])")
    /// );
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_length_arg = true;
    ///
    /// let demangled = demangle("simpler_array__FPA41_A24_Ci", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("simpler_array(int const (*)[42][25])")
    /// );
    /// ```
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
    /// This is just another c++filt compatibility setting.
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

    /// Emit an space between a comma and an ellipsis (`...`) in the argument
    /// list.
    ///
    /// This is just another c++filt compatibility setting.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.ellipsis_emit_space_after_comma = false;
    ///
    /// let demangled = demangle("Printf__7ConsolePce", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("Console::Printf(char *,...)")
    /// );
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.ellipsis_emit_space_after_comma = true;
    ///
    /// let demangled = demangle("Printf__7ConsolePce", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("Console::Printf(char *, ...)")
    /// );
    /// ```
    pub ellipsis_emit_space_after_comma: bool,

    /// If enabled, emit `__int128_t` and `__uint128_t` types instead of
    /// `int128_t` and `unsigned int128_t`.
    ///
    /// The former is valid syntax in g++ for this GNU integer extension type,
    /// while the latter is the syntax used by c++filt, but not accepted by g++.
    ///
    /// This is just another c++filt compatibility setting.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.fix_extension_int = false;
    ///
    /// let demangled = demangle("testing_func__FRCI80", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("testing_func(int128_t const &)")
    /// );
    /// let demangled = demangle("testing_func__FRCUI80", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("testing_func(unsigned int128_t const &)")
    /// );
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.fix_extension_int = true;
    ///
    /// let demangled = demangle("testing_func__FRCI80", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("testing_func(__int128_t const &)")
    /// );
    /// let demangled = demangle("testing_func__FRCUI80", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("testing_func(__uint128_t const &)")
    /// );
    /// ```
    pub fix_extension_int: bool,

    /// If enabled, emit proper syntax for arrays as return types in templated
    /// functions.
    ///
    /// Disabling this option make it mimic the c++filt behavior for arrays in
    /// return position, which is not valid C++ but is simpler to read.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_in_return_position = false;
    ///
    /// let demangled = demangle("an_array__H1Zi_X01_PA3_f", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("float (*)[3] an_array<int>(int)")
    /// );
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_in_return_position = true;
    ///
    /// let demangled = demangle("an_array__H1Zi_X01_PA3_f", &config);
    /// assert_eq!(
    ///     demangled.as_deref(),
    ///     Ok("float (*an_array<int>(int))[3]")
    /// );
    /// ```
    pub fix_array_in_return_position: bool,
}

impl DemangleConfig {
    /// The default config mimics the default (rather questionable) c++filt's
    /// behavior, including what may be considered c++filt bugs.
    ///
    /// Note this default may change in a future version.
    pub const fn new() -> Self {
        Self::new_mimic_cfilt()
    }

    /// The default config mimics the default (rather questionable) c++filt's
    /// behavior, including what may be considered c++filt bugs.
    pub const fn new_mimic_cfilt() -> Self {
        Self {
            preserve_namespaced_global_constructor_bug: true,
            fix_array_length_arg: false,
            demangle_global_keyed_frames: false,
            ellipsis_emit_space_after_comma: false,
            fix_extension_int: false,
            fix_array_in_return_position: false,
        }
    }

    /// Avoid using any option that mimics c++filt faults.
    pub const fn new_no_cfilt_mimics() -> Self {
        Self {
            preserve_namespaced_global_constructor_bug: false,
            fix_array_length_arg: true,
            demangle_global_keyed_frames: true,
            ellipsis_emit_space_after_comma: true,
            fix_extension_int: true,
            fix_array_in_return_position: true,
        }
    }
}

impl Default for DemangleConfig {
    fn default() -> Self {
        Self::new()
    }
}
