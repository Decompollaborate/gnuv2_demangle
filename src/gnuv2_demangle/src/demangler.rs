/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, boxed::Box};

use crate::{
    dem_branch::{
        dem_destructor::demangle_destructor,
        dem_free_function::demangle_free_function,
        dem_in_function_static_initialized::demangle_in_function_static_initialized,
        dem_method::demangle_method,
        dem_namespaced_function::demangle_namespaced_function,
        dem_namespaced_global::demangle_namespaced_global,
        namespaced_type_class_method::demangle_namespaced_type_class_method,
        dem_templated_function::demangle_templated_function,
        dem_type_info::{demangle_type_info_function, demangle_type_info_node},
        dem_virtual_tables::demangle_virtual_table,
    },
    dem_leaf::{
        dem::demangle_method_qualifier, dem_arg::demangle_argument,
        dem_arg_list::demangle_argument_list, dem_namespaced_type::demangle_namespaced_type,
    },
    remainer::Remaining,
    str_cutter::StrCutter,
    tree::{
        ArgVec, Demangled, DemangledArg, GlobalSymKeyed, GlobalSymKeyedKind, GlobalSymKeyedSym,
        MethodQualifier, NamespacedType, StaticSuffix,
    },
    DemangleConfig, DemangleError,
};

/// Demangle a symbol.
///
/// See [`DemangleConfig`] for tweaking the demangled output.
///
/// # Examples
///
/// ```
/// use gnuv2_demangle::{demangle, DemangleConfig};
///
/// # fn main() -> Result<(), gnuv2_demangle::DemangleError<'static>> {
/// let config = DemangleConfig::new();
///
/// let demangled = demangle("_$_5tName", &config)?;
/// assert_eq!(
///     demangled.to_string().as_str(),
///     "tName::~tName(void)",
/// );
///
/// let demangled = demangle("a_function__Q35silly8my_thing17another_namespacefffi", &config)?;
/// assert_eq!(
///     demangled.to_string().as_str(),
///     "silly::my_thing::another_namespace::a_function(float, float, float, int)",
/// );
/// # Ok(())
/// # }
/// ```
pub fn demangle<'s>(
    sym: &'s str,
    config: &DemangleConfig,
) -> Result<Demangled<'s>, DemangleError<'s>> {
    if !sym.is_ascii() {
        Err(DemangleError::NonAscii)
    } else {
        // GCC lets users change the default marker ('$') for compatibility
        // with other toolchains that do not accept '$' in symbol names.
        let cplus_marker = determine_cplus_marker(sym);

        demangle_impl(sym, config, cplus_marker, true)
    }
}

fn determine_cplus_marker(sym: &str) -> char {
    let tentative = if sym.starts_with("_$_") {
        // destructor
        Some('$')
    } else if sym.starts_with("_._") {
        // destructor
        Some('.')
    } else if sym.starts_with("_GLOBAL_$") {
        // global sym keyed
        Some('$')
    } else if sym.starts_with("_GLOBAL_.") {
        // global sym keyed
        Some('.')
    } else if sym
        .c_strip_prefix_3str_and_pat("_", '$', "tmp_", |c| {
            matches!(c.chars().next(), Some('0'..='9'))
        })
        .is_some()
    {
        // in function static initialized
        Some('$')
    } else if sym
        .c_strip_prefix_3str_and_pat("_", '.', "tmp_", |c| {
            matches!(c.chars().next(), Some('0'..='9'))
        })
        .is_some()
    {
        // in function static initialized
        Some('.')
    } else if sym.starts_with("_vt$") {
        // virtual table
        Some('$')
    } else if sym.starts_with("_vt.") {
        // virtual table
        Some('.')
    } else {
        None
    };

    tentative.unwrap_or_else(|| {
        let namespaced_global =
            sym.starts_with('_') && sym[1..].starts_with(|c| matches!(c, 't' | 'Q' | '1'..='9'));

        if namespaced_global && sym.contains('$') {
            // namespaced global
            '$'
        } else if namespaced_global && sym.contains('.') {
            // namespaced global
            '.'
        } else {
            // Unable to determine, fallback
            '$'
        }
    })
}

fn demangle_impl<'s>(
    sym: &'s str,
    config: &DemangleConfig,
    cplus_marker: char,
    allow_global_sym_keyed: bool,
) -> Result<Demangled<'s>, DemangleError<'s>> {
    if let Some(s) = sym.c_strip_prefix_3chars('_', cplus_marker, '_') {
        demangle_destructor(config, s).map(Demangled::Destructor)
    } else if let Some(s) = sym.strip_prefix("__") {
        demangle_special(config, s, sym)
    } else if let Some(s) =
        sym.c_cond_and_strip_prefix_and_char(allow_global_sym_keyed, "_GLOBAL_", cplus_marker)
    {
        demangle_global_sym_keyed(config, s, cplus_marker, sym)
    } else if let Some(s) = sym.c_strip_prefix_3str_and_pat("_", cplus_marker, "tmp_", |c| {
        matches!(c.chars().next(), Some('0'..='9'))
    }) {
        demangle_in_function_static_initialized(config, s)
            .map(Demangled::InFunctionStaticInitialized)
    } else {
        demangle_impl_failables(sym, config, cplus_marker)
    }
}

fn demangle_impl_failables<'s>(
    sym: &'s str,
    config: &DemangleConfig,
    cplus_marker: char,
) -> Result<Demangled<'s>, DemangleError<'s>> {
    // Some of the checks here can overlap and produce false positives, so if
    // one fails then try again with the next one, over and over.

    let leading_error = None;

    // Look up for the first appareance of something like `__F`, `__t`, `__H`, etc. and just use that
    let leading_error = if let Some((sym_name, the_rest, c)) = sym
        .c_split2_r_starts_with("__", |c| {
            matches!(c, 'F' | '1'..='9' | 'C' | 't' | 'H' | 'Q')
        }) {
        // All the cases here should be the same as the match above.
        match c {
            'F' => match demangle_free_function(config, sym_name, &the_rest[1..]) {
                Ok(d) => return Ok(Demangled::FreeFunction(d)),
                Err(e) => leading_error.or(Some(e)),
            },
            '1'..='9' | 'C' | 't' => match demangle_method(config, sym_name, the_rest) {
                Ok(d) => return Ok(Demangled::Method(d)),
                Err(e) => leading_error.or(Some(e)),
            },
            'H' => match demangle_templated_function(config, sym_name, &the_rest[1..]) {
                Ok(d) => return Ok(Demangled::TemplatedFunction(d)),
                Err(e) => leading_error.or(Some(e)),
            },
            'Q' => match demangle_namespaced_function(config, sym_name, &the_rest[1..]) {
                Ok(d) => return Ok(Demangled::NamespacedFunction(d)),
                Err(e) => leading_error.or(Some(e)),
            },
            _ => unreachable!(),
        }
    } else {
        None
    };

    let leading_error = if let Some(sym) = sym.strip_prefix("_vt") {
        match demangle_virtual_table(config, sym, cplus_marker) {
            Ok(d) => return Ok(Demangled::VirtualTable(d)),
            Err(e) => leading_error.or(Some(e)),
        }
    } else {
        leading_error
    };

    let leading_error = if let Some((s, name)) = sym.c_split2_char(cplus_marker) {
        match demangle_namespaced_global(config, s, name) {
            Ok(d) => return Ok(Demangled::NamespacedGlobal(d)),
            Err(e) => leading_error.or(Some(e)),
        }
    } else {
        leading_error
    };

    // Raise the previous error if any exists.
    Err(leading_error.unwrap_or(DemangleError::NotMangled))
}

// TODO: remove
enum SpecialDiscriminator {
    OperatorFree,
    OperatorMethod,
}

fn demangle_special<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_sym: &'s str,
) -> Result<Demangled<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (remaining, class_name, method_name, suffix, discriminator) = if let Some(remaining) =
        s.strip_prefix("tf")
    {
        return demangle_type_info_function(config, remaining).map(Demangled::TypeInfoFunction);
    } else if let Some(remaining) = s.strip_prefix("ti") {
        return demangle_type_info_node(config, remaining).map(Demangled::TypeInfoNode);
    } else if let Some(nt) = demangle_namespaced_type(
        config,
        s,
        allow_array_fixup,
        DemangleError::InvalidClassNameOnConstructor,
    )? {
        let Remaining {
            r,
            d: namespaced_type,
        } = nt;

        return demangle_namespaced_type_class_method(
            config,
            r,
            namespaced_type,
            allow_array_fixup,
        )
        .map(Demangled::NamespacedTypeClassMethod);
    } else {
        let end_index = s.find("__").ok_or(DemangleError::InvalidSpecialMethod(s))?;
        let op = &s[..end_index];

        // Skip the underscore
        let remaining = &s[end_index + 2..];

        let method_name = match op {
            // Memory
            "nw" => Cow::from("operator new"),
            "dl" => Cow::from("operator delete"),
            "vn" => Cow::from("operator new []"),
            "vd" => Cow::from("operator delete []"),

            // Comparison
            "eq" => Cow::from("operator=="),
            "ne" => Cow::from("operator!="),
            "lt" => Cow::from("operator<"),
            "gt" => Cow::from("operator>"),
            "le" => Cow::from("operator<="),
            "ge" => Cow::from("operator>="),

            // Assignment
            "as" => Cow::from("operator="),
            "apl" => Cow::from("operator+="),
            "ami" => Cow::from("operator-="),
            "aml" => Cow::from("operator*="),
            "adv" => Cow::from("operator/="),
            "amd" => Cow::from("operator%="),
            "aer" => Cow::from("operator^="),
            "aad" => Cow::from("operator&="),
            "aor" => Cow::from("operator|="),
            "als" => Cow::from("operator<<="),
            "ars" => Cow::from("operator>>="),

            // Bitwise
            "er" => Cow::from("operator^"),
            "ad" => Cow::from("operator&"),
            "or" => Cow::from("operator|"),
            "ls" => Cow::from("operator<<"),
            "rs" => Cow::from("operator>>"),
            "co" => Cow::from("operator~"),

            // Increment/Decrement
            "pp" => Cow::from("operator++"),
            "mm" => Cow::from("operator--"),

            // Logical
            "aa" => Cow::from("operator&&"),
            "oo" => Cow::from("operator||"),
            "nt" => Cow::from("operator!"),

            // Member access
            "vc" => Cow::from("operator[]"),
            "rf" => Cow::from("operator->"),
            "rm" => Cow::from("operator->*"),

            // Arithmetic
            "pl" => Cow::from("operator+"),
            "mi" => Cow::from("operator-"),
            "ml" => Cow::from("operator*"),
            "dv" => Cow::from("operator/"),
            "md" => Cow::from("operator%"),

            // Other
            "cl" => Cow::from("operator()"),
            "cm" => Cow::from("operator, "),

            _ => {
                if let Some(cast) = op.strip_prefix("op") {
                    let (remaining, DemangledArg::Plain(typ, array_qualifiers)) =
                        demangle_argument(
                            config,
                            cast,
                            &ArgVec::new(config, None),
                            &ArgVec::new(config, None),
                            allow_array_fixup,
                        )?
                    else {
                        return Err(DemangleError::UnrecognizedSpecialMethod(op));
                    };
                    if !remaining.is_empty() {
                        return Err(DemangleError::MalformedCastOperatorOverload(remaining));
                    }

                    Cow::from(format!("operator {typ}{array_qualifiers}"))
                } else {
                    return {
                        // This may be a plain function that got confused with a
                        // special symbol, so try to decode as a function instead.
                        if let Some((func_name, args)) = full_sym.c_split2("__F") {
                            demangle_free_function(config, func_name, args)
                                .map(Demangled::FreeFunction)
                        } else if let Some((incomplete_method_name, class_and_args, _c)) =
                            s.c_split2_r_starts_with("__", |c| matches!(c, '1'..='9' | 'C' | 't'))
                        {
                            // split `s` instead of `full_sym` to skip over the
                            // first `__`,
                            // if that check passes, then recover the actual
                            // method name, including the initial `__`, by
                            // using the length of the `incomplete_method_name`
                            // to slice the `full_sym`.

                            let method_name = &full_sym[..incomplete_method_name.len() + 2];
                            demangle_method(config, method_name, class_and_args)
                                .map(Demangled::Method)
                        } else if let Some((func_name, s)) = full_sym.c_split2("__H") {
                            demangle_templated_function(config, func_name, s)
                                .map(Demangled::TemplatedFunction)
                        } else {
                            Err(DemangleError::UnrecognizedSpecialMethod(op))
                        }
                    };
                }
            }
        };

        if let Some(remaining) = remaining.strip_prefix('F') {
            (
                remaining,
                None,
                method_name,
                MethodQualifier::No,
                SpecialDiscriminator::OperatorFree,
            )
        } else {
            let Remaining {
                r: remaining,
                d: suffix,
            } = demangle_method_qualifier(remaining);

            let Remaining {
                r: remaining,
                d:
                    NamespacedType {
                        namespace,
                        type_name: _,
                    },
            } = demangle_namespaced_type(
                config,
                remaining,
                allow_array_fixup,
                DemangleError::InvalidClassNameOnOperator,
            )?
            .ok_or(DemangleError::InvalidTypeNameForOperator(remaining))?;

            (
                remaining,
                Some(namespace),
                method_name,
                suffix,
                SpecialDiscriminator::OperatorMethod,
            )
        }
    };

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            class_name.as_ref(),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    let out = if let Some(class_name) = class_name {
        format!("{class_name}::{method_name}{static_suffix}({argument_list}){suffix}")
    } else {
        format!("{method_name}{static_suffix}({argument_list}){suffix}")
    };
    Ok(match discriminator {
        SpecialDiscriminator::OperatorFree => Demangled::OperatorFree(out),
        SpecialDiscriminator::OperatorMethod => Demangled::OperatorMethod(out),
    })
}

fn demangle_global_sym_keyed<'s>(
    config: &DemangleConfig,
    s: &'s str,
    cplus_marker: char,
    full_sym: &'s str,
) -> Result<Demangled<'s>, DemangleError<'s>> {
    let (remaining, which) = if let Some(r) = s.strip_prefix("I") {
        (r, GlobalSymKeyedKind::Constructors)
    } else if let Some(r) = s.strip_prefix("D") {
        (r, GlobalSymKeyedKind::Destructors)
    } else if let Some(r) = s.strip_prefix("F") {
        if config.demangle_global_keyed_frames {
            (r, GlobalSymKeyedKind::Frames)
        } else {
            // !HACK(c++filt): c++filt does not recognize `_GLOBAL_$F$`, so it
            // !tries to demangle it as anything else.
            return demangle_impl(full_sym, config, cplus_marker, false);
        }
    } else {
        return Err(DemangleError::InvalidGlobalSymKeyed(s));
    };

    let Some(remaining) = remaining.strip_prefix(cplus_marker) else {
        return Err(DemangleError::InvalidGlobalSymKeyed(s));
    };

    let demangled_sym = demangle_impl(remaining, config, cplus_marker, false);
    if !config.fix_namespaced_global_constructor_bug
        && which.is_constructor()
        && remaining.starts_with("__Q")
    {
        // !HACK(c++filt): Seems like c++filt has a bug where it won't output
        // !the "global constructors keyed to " prefix for namespaced functions
        return demangled_sym;
    }

    let actual_sym = demangled_sym.map_or_else(
        |_| GlobalSymKeyedSym::Raw(remaining),
        |d| GlobalSymKeyedSym::Demangled(Box::new(d)),
    );

    let gsk = GlobalSymKeyed::new(which, actual_sym);
    Ok(Demangled::GlobalSymKeyed(gsk))
}
