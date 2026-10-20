/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/// Tweak how a symbol should be disassembled.
///
/// The constructors provide sensible defaults, so there's usually no need to
/// override each option.
///
/// Refer to each option to see what it does and examples.
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
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_namespaced_global_constructor_bug = false;
    ///
    /// let demangled = demangle("_GLOBAL_$I$__Q210Scenegraph10Scenegraph", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "Scenegraph::Scenegraph::Scenegraph(void)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_namespaced_global_constructor_bug = true;
    ///
    /// let demangled = demangle("_GLOBAL_$I$__Q210Scenegraph10Scenegraph", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "global constructors keyed to Scenegraph::Scenegraph::Scenegraph(void)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fix_namespaced_global_constructor_bug: bool,

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
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_length_arg = false;
    ///
    /// let demangled = demangle("simpler_array__FPA41_A24_Ci", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "simpler_array(int const (*)[41][24])",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_length_arg = true;
    ///
    /// let demangled = demangle("simpler_array__FPA41_A24_Ci", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "simpler_array(int const (*)[42][25])",
    /// );
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.demangle_global_keyed_frames = false;
    ///
    /// let demangled = demangle("_GLOBAL_$F$__7istreamiP9streambufP7ostream", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "istream::_GLOBAL_$F$(int, streambuf *, ostream *)",
    /// );
    /// let demangled = demangle("_GLOBAL_$F$__default_terminate", &config);
    /// assert!(
    ///     demangled.is_err()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.demangle_global_keyed_frames = true;
    ///
    /// let demangled = demangle("_GLOBAL_$F$__7istreamiP9streambufP7ostream", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "global frames keyed to istream::istream(int, streambuf *, ostream *)",
    /// );
    /// let demangled = demangle("_GLOBAL_$F$__default_terminate", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "global frames keyed to __default_terminate",
    /// );
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.ellipsis_emit_space_after_comma = false;
    ///
    /// let demangled = demangle("Printf__7ConsolePce", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "Console::Printf(char *,...)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.ellipsis_emit_space_after_comma = true;
    ///
    /// let demangled = demangle("Printf__7ConsolePce", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "Console::Printf(char *, ...)",
    /// );
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_extension_int = false;
    ///
    /// let demangled = demangle("testing_func__FRCI80", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "testing_func(int128_t const &)",
    /// );
    /// let demangled = demangle("testing_func__FRCUI80", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "testing_func(unsigned int128_t const &)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_extension_int = true;
    ///
    /// let demangled = demangle("testing_func__FRCI80", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "testing_func(__int128_t const &)",
    /// );
    /// let demangled = demangle("testing_func__FRCUI80", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "testing_func(__uint128_t const &)",
    /// );
    /// # Ok(())
    /// # }
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
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_in_return_position = false;
    ///
    /// let demangled = demangle("an_array__H1Zi_X01_PA3_f", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "float (*)[3] an_array<int>(int)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_array_in_return_position = true;
    ///
    /// let demangled = demangle("an_array__H1Zi_X01_PA3_f", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "float (*an_array<int>(int))[3]",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fix_array_in_return_position: bool,

    /// If enabled, emit proper syntax for return types of function pointers in
    /// template lists.
    ///
    /// Disabling this option make it mimic the c++filt behavior for function
    /// pointers in template lists, which is not valid C++ but is simpler to
    /// read.
    ///
    /// The c++filt behavior also omits the return type of the function pointer
    /// while this option does explicitly shows it.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_function_pointers_in_template_lists = false;
    ///
    /// let demangled = demangle("alloc__t5Table1PFUi_Pv16DefaultFunc__FUiUi", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "Table<&DefaultFunc(unsigned int)>::alloc(unsigned int)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.fix_function_pointers_in_template_lists = true;
    ///
    /// let demangled = demangle("alloc__t5Table1PFUi_Pv16DefaultFunc__FUiUi", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "Table<(void *(*)(unsigned int)) &DefaultFunc>::alloc(unsigned int)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fix_function_pointers_in_template_lists: bool,

    /// Handle the static numeric suffix generated by g++ used to avoid name
    /// clashes.
    ///
    /// This pattern is specifically a trailing dot followed by an arbitrarily
    /// long number.
    ///
    /// This is just another c++filt compatibility setting.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig, DemangleError};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.demangle_static_suffix = false;
    ///
    /// let demangled = demangle("inner_func__Q216outer_func__Fv.0_7Private.3", &config);
    /// assert_eq!(
    ///     demangled,
    ///     Err(DemangleError::UnknownType('.', ".3")),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.demangle_static_suffix = true;
    ///
    /// let demangled = demangle("inner_func__Q216outer_func__Fv.0_7Private.3", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "outer_func.0(void)::Private::inner_func.3(void)",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub demangle_static_suffix: bool,

    /// Handle demangling the "in-function-static initialized" compiler
    /// generated symbols
    ///
    /// Those are compiler-generated symbols used to track if an in-function
    /// static symbol has been initialized or not. Used only when the
    /// initializer is a non constant value, like a function call.
    ///
    /// This is a made up convention since this doesn't seem to be a common
    /// thing to demangle at all.
    ///
    /// This is just another c++filt compatibility setting.
    ///
    /// # Examples
    ///
    /// Turning off this setting (mimicking c++filt behavior):
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig, DemangleError};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.demangle_in_function_static_initialized = false;
    ///
    /// let demangled = demangle("_$tmp_0.7", &config);
    /// assert_eq!(
    ///     demangled,
    ///     Err(DemangleError::InFunctionStaticInitializedDisabled("0.7")),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The setting turned on:
    ///
    /// ```
    /// use gnuv2_demangle::{demangle, DemangleConfig};
    ///
    /// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
    /// let mut config = DemangleConfig::new();
    /// config.demangle_in_function_static_initialized = true;
    ///
    /// let demangled = demangle("_$tmp_0.7", &config)?;
    /// assert_eq!(
    ///     demangled.to_string().as_str(),
    ///     "in-function-static initialized 0 7",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub demangle_in_function_static_initialized: bool,
}

impl DemangleConfig {
    /// The default configuration.
    #[must_use]
    #[inline]
    #[track_caller]
    pub const fn new() -> Self {
        Self::new_g2dem()
    }

    /// Use improved output and valid C++ syntax whenever possible.
    #[must_use]
    #[inline]
    #[track_caller]
    pub const fn new_g2dem() -> Self {
        Self {
            fix_namespaced_global_constructor_bug: true,
            fix_array_length_arg: true,
            demangle_global_keyed_frames: true,
            ellipsis_emit_space_after_comma: true,
            fix_extension_int: true,
            fix_array_in_return_position: true,
            fix_function_pointers_in_template_lists: true,
            demangle_static_suffix: true,
            demangle_in_function_static_initialized: true,
        }
    }

    /// Mimics the (rather questionable) c++filt's behavior, including what may
    /// be considered c++filt bugs.
    ///
    /// Useful for validating demangling against c++filt.
    #[must_use]
    #[inline]
    #[track_caller]
    pub const fn new_cfilt() -> Self {
        Self {
            fix_namespaced_global_constructor_bug: false,
            fix_array_length_arg: false,
            demangle_global_keyed_frames: false,
            ellipsis_emit_space_after_comma: false,
            fix_extension_int: false,
            fix_array_in_return_position: false,
            fix_function_pointers_in_template_lists: false,
            demangle_static_suffix: false,
            demangle_in_function_static_initialized: false,
        }
    }
}

impl Default for DemangleConfig {
    fn default() -> Self {
        Self::new()
    }
}
