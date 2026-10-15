/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::Write;

use crate::{
    dem_namespaced_type::demangle_namespaced_type,
    dem_other::{demangle_static_suffix, StaticSuffix},
    tree::{NamespaceKind, NamespacedType},
    DemangleConfig, DemangleError,
};

use crate::{
    dem::demangle_method_qualifier,
    dem_arg::{demangle_argument, DemangledArg},
    dem_arg_list::{demangle_argument_list, demangle_argument_list_impl, ArgVec},
    dem_namespace::demangle_namespaces,
    dem_template::demangle_template_with_return_type,
    remainer::Remaining,
    str_cutter::StrCutter,
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
/// let config = DemangleConfig::new();
///
/// let demangled = demangle("_$_5tName", &config);
/// assert_eq!(
///     demangled.as_deref(),
///     Ok("tName::~tName(void)")
/// );
///
/// let demangled = demangle("a_function__Q35silly8my_thing17another_namespacefffi", &config);
/// assert_eq!(
///     demangled.as_deref(),
///     Ok("silly::my_thing::another_namespace::a_function(float, float, float, int)")
/// );
/// ```
pub fn demangle<'s>(sym: &'s str, config: &DemangleConfig) -> Result<String, DemangleError<'s>> {
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
) -> Result<String, DemangleError<'s>> {
    if let Some(s) = sym.c_strip_prefix_3chars('_', cplus_marker, '_') {
        demangle_destructor(config, s)
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
    } else {
        demangle_impl_failables(sym, config, cplus_marker)
    }
}

fn demangle_impl_failables<'s>(
    sym: &'s str,
    config: &DemangleConfig,
    cplus_marker: char,
) -> Result<String, DemangleError<'s>> {
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
                Ok(d) => return Ok(d),
                Err(e) => leading_error.or(Some(e)),
            },
            '1'..='9' | 'C' | 't' => match demangle_method(config, sym_name, the_rest) {
                Ok(d) => return Ok(d),
                Err(e) => leading_error.or(Some(e)),
            },
            'H' => match demangle_templated_function(config, sym_name, &the_rest[1..]) {
                Ok(d) => return Ok(d),
                Err(e) => leading_error.or(Some(e)),
            },
            'Q' => match demangle_namespaced_function(config, sym_name, &the_rest[1..]) {
                Ok(d) => return Ok(d),
                Err(e) => leading_error.or(Some(e)),
            },
            _ => unreachable!(),
        }
    } else {
        None
    };

    let leading_error = if let Some(sym) = sym.strip_prefix("_vt") {
        match demangle_virtual_table(config, sym, cplus_marker) {
            Ok(d) => return Ok(d),
            Err(e) => leading_error.or(Some(e)),
        }
    } else {
        leading_error
    };

    let leading_error = if let Some((s, name)) = sym.c_split2_char(cplus_marker) {
        match demangle_namespaced_global(config, s, name) {
            Ok(d) => return Ok(d),
            Err(e) => leading_error.or(Some(e)),
        }
    } else {
        leading_error
    };

    // Raise the previous error if any exists.
    Err(leading_error.unwrap_or(DemangleError::NotMangled))
}

fn demangle_destructor<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;

    let Remaining {
        r,
        d: NamespacedType {
            namespace,
            type_name,
        },
    } = demangle_namespaced_type(
        config,
        s,
        allow_array_fixup,
        DemangleError::InvalidClassNameOnDestructor,
    )?
    .ok_or(DemangleError::InvalidTypeNameForDestructor(s))?;

    let Remaining {
        r,
        d: static_suffix,
    } = demangle_static_suffix(config, r)?;

    if r.is_empty() {
        Ok(format!(
            "{namespace}{static_suffix}::~{type_name}{static_suffix}(void)"
        ))
    } else {
        Err(DemangleError::TrailingDataOnDestructor(r))
    }
}

fn demangle_special<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_sym: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (remaining, class_name, method_name, suffix) = if let Some(remaining) = s.strip_prefix("tf")
    {
        return demangle_type_info_function(config, remaining);
    } else if let Some(remaining) = s.strip_prefix("ti") {
        return demangle_type_info_node(config, remaining);
    } else if let Some(nt) = demangle_namespaced_type(
        config,
        s,
        allow_array_fixup,
        DemangleError::InvalidClassNameOnConstructor,
    )? {
        let Remaining {
            r,
            d: NamespacedType {
                namespace,
                type_name,
            },
        } = nt;

        (r, Some(namespace), Cow::from(type_name), "")
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
                        } else if let Some((func_name, s)) = full_sym.c_split2("__H") {
                            demangle_templated_function(config, func_name, s)
                        } else {
                            Err(DemangleError::UnrecognizedSpecialMethod(op))
                        }
                    };
                }
            }
        };

        if let Some(remaining) = remaining.strip_prefix('F') {
            (remaining, None, method_name, "")
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

            (remaining, Some(namespace), method_name, suffix)
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
    Ok(out)
}

fn demangle_free_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (argument_list, static_suffix) = demangle_argument_list(
        config,
        args,
        None,
        &ArgVec::new(config, None),
        allow_array_fixup,
    )?;

    Ok(format!("{func_name}{static_suffix}({argument_list})"))
}

fn demangle_method<'s>(
    config: &DemangleConfig,
    method_name: &'s str,
    class_and_args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;
    let Remaining {
        r: remaining,
        d: suffix,
    } = demangle_method_qualifier(class_and_args);

    let Remaining {
        r: remaining,
        d: NamespacedType {
            namespace,
            type_name: _,
        },
    } = demangle_namespaced_type(
        config,
        remaining,
        allow_array_fixup,
        DemangleError::InvalidClassNameOnMethod,
    )?
    .ok_or(DemangleError::InvalidTypeNameForMethod(remaining))?;

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            Some(&namespace),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    Ok(format!(
        "{namespace}::{method_name}{static_suffix}({argument_list}){suffix}"
    ))
}

/// Templated functions and methods.
///
/// A templated method is templated individually, it doesn't matter if the
/// class it comes from is templated or not.
fn demangle_templated_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    // Arrays do need to be fixed up if it appears in the template list, but
    // not in the rest of the definition.
    let allow_array_fixup = true;
    let (remaining, template_args, typ) =
        demangle_template_with_return_type(config, s, allow_array_fixup)?;
    let allow_array_fixup = false;

    let Remaining {
        r: remaining,
        d: suffix,
    } = demangle_method_qualifier(remaining);

    let (remaining, typ) = if let Some(typ) = typ {
        (remaining, Some(typ.namespace))
    } else if let Some(nt) = demangle_namespaced_type(
        config,
        remaining,
        allow_array_fixup,
        DemangleError::InvalidNamespaceOnTemplatedFunction,
    )? {
        let Remaining { r, d: nt } = nt;
        (r, Some(nt.namespace))
    } else {
        (remaining, None)
    };

    // - First there's an optional namespace used in template specializations.
    // - Then there's the optional argument list, which may be omitted if there
    //   are no parameters.
    // - Finally the return type.

    // Demangle the specialization namespace
    let (remaining, specialization_namespace) = if let Some(r) = remaining.strip_prefix('_') {
        let (r, DemangledArg::Plain(specialization_namespace, array_qualifiers)) =
            demangle_argument(
                config,
                r,
                &ArgVec::new(config, typ.as_ref()),
                &template_args,
                allow_array_fixup,
            )?
        else {
            return Err(DemangleError::MalformedTemplatedSpecializationInvalidNamespace(r));
        };

        (r, Some((specialization_namespace, array_qualifiers)))
    } else {
        (remaining, None)
    };

    // Demangle the argument list
    let (remaining, argument_list) = demangle_argument_list_impl(
        config,
        remaining,
        typ.as_ref(),
        &template_args,
        false,
        allow_array_fixup,
    )?;

    // Demangle the return type
    let (specialization_namespace, return_type, array_qualifiers) =
        if let Some(r) = remaining.strip_prefix('_') {
            let (r, DemangledArg::Plain(ret_type, array_qualifiers)) = demangle_argument(
                config,
                r,
                &ArgVec::new(config, typ.as_ref()),
                &template_args,
                allow_array_fixup,
            )?
            else {
                return Err(DemangleError::MalformedTemplateWithReturnTypeMissingReturnType(r));
            };

            if !r.is_empty() {
                return Err(
                    DemangleError::TrailingDataAfterReturnTypeOfMalformedTemplateWithReturnType(r),
                );
            }
            (specialization_namespace, ret_type, array_qualifiers)
        } else if let Some((actual_return_type, array_qualifiers)) = specialization_namespace {
            // If there's no argument list and this symbol is not a template
            // specialization inside a namespace then we mistakenly consumed the
            // return type as the specialization_namespace

            if !remaining.is_empty() {
                return Err(
                    DemangleError::TrailingDataAfterReturnTypeOfTemplatedSpecialization(remaining),
                );
            }

            (None, actual_return_type, array_qualifiers)
        } else {
            return Err(DemangleError::MalformedTemplateWithReturnTypeMissingReturnType(remaining));
        };

    let template_args = template_args.join();
    let formated_template_args = if template_args.ends_with('>') {
        format!("<{} >", template_args)
    } else {
        format!("<{}>", template_args)
    };
    let argument_list = argument_list.join();

    let mut out = return_type;
    if let Some((specialization_namespace, _array_qualifiers)) = specialization_namespace {
        out.push(' ');
        out.push_str(&specialization_namespace);
    }
    if let Some(array_qualifiers) = array_qualifiers.as_option() {
        if config.fix_array_in_return_position {
            out.push_str(" (");
            out.push_str(&array_qualifiers.inner_post_qualifiers);
        } else {
            out.push_str(&array_qualifiers.to_string());
            out.push(' ');
        }
    } else {
        out.push(' ');
    }
    if let Some(typ) = typ {
        out.push_str(&typ.to_string());
        out.push_str("::");
    }
    out.push_str(func_name);
    out.push_str(&formated_template_args);
    out.push('(');
    out.push_str(&argument_list);
    out.push(')');
    out.push_str(suffix);
    if let Some(array_qualifiers) = array_qualifiers.as_option() {
        if config.fix_array_in_return_position {
            out.push(')');
            out.push_str(&array_qualifiers.arrays);
        }
    }

    Ok(out)
}

fn demangle_namespaced_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;

    let Remaining {
        r: remaining,
        d: (namespaces, _trailing_namespace),
    } = demangle_namespaces(config, s, &ArgVec::new(config, None), allow_array_fixup)?;
    let namespaces = NamespaceKind::Namespace(namespaces);

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            Some(&namespaces),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    let out = format!("{namespaces}::{func_name}{static_suffix}({argument_list})");
    Ok(out)
}

fn demangle_type_info_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;

    if let (remaining, DemangledArg::Plain(demangled_type, array_qualifiers)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &ArgVec::new(config, None),
        allow_array_fixup,
    )? {
        if remaining.is_empty() {
            Ok(format!(
                "{demangled_type}{array_qualifiers} type_info function"
            ))
        } else {
            Err(DemangleError::TrailingDataOnTypeInfoFunction(remaining))
        }
    } else {
        Err(DemangleError::InvalidTypeOnTypeInfoFunction(s))
    }
}

fn demangle_type_info_node<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;

    if let (remaining, DemangledArg::Plain(demangled_type, array_qualifiers)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &ArgVec::new(config, None),
        allow_array_fixup,
    )? {
        if remaining.is_empty() {
            Ok(format!("{demangled_type}{array_qualifiers} type_info node"))
        } else {
            Err(DemangleError::TrailingDataOnTypeInfoNode(remaining))
        }
    } else {
        Err(DemangleError::InvalidTypeOnTypeInfoNode(s))
    }
}

fn demangle_virtual_table<'s>(
    config: &DemangleConfig,
    s: &'s str,
    cplus_marker: char,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;
    let mut remaining = s;
    let mut stuff = Vec::new();

    while !remaining.is_empty() {
        remaining = remaining
            .strip_prefix(cplus_marker)
            .ok_or(DemangleError::VTableMissingDollarSeparator(remaining))?;

        let Remaining {
            r,
            d:
                NamespacedType {
                    namespace,
                    type_name: _,
                },
        } = demangle_namespaced_type(
            config,
            remaining,
            allow_array_fixup,
            DemangleError::InvalidClassNameOnVirtualTable,
        )?
        .ok_or(DemangleError::InvalidTypeNameForVirtualTable(remaining))?;

        stuff.push(namespace);
        remaining = r;
    }

    let mut buf = String::new();
    {
        let mut iter = stuff.into_iter();

        // TODO: get rid of these unwraps
        if let Some(ns) = iter.next() {
            write!(buf, "{ns}").unwrap();
        }
        for ns in iter {
            write!(buf, "::{ns}").unwrap();
        }
    }
    Ok(format!("{buf} virtual table"))
}

fn demangle_namespaced_global<'s>(
    config: &DemangleConfig,
    s: &'s str,
    name: &'s str,
) -> Result<String, DemangleError<'s>> {
    let allow_array_fixup = true;

    let Some(remaining) = s.strip_prefix('_') else {
        return Err(DemangleError::InvalidNamespacedGlobal(s, name));
    };

    let Remaining {
        r,
        d: NamespacedType {
            namespace: space,
            type_name: _,
        },
    } = demangle_namespaced_type(
        config,
        remaining,
        allow_array_fixup,
        DemangleError::InvalidNamespaceOnNamespacedGlobal,
    )?
    .ok_or(DemangleError::InvalidTypeNameForNamespacedGlobal(remaining))?;

    if !r.is_empty() {
        return Err(DemangleError::TrailingDataOnNamespacedGlobal(r));
    }

    Ok(format!("{space}::{name}"))
}

fn demangle_global_sym_keyed<'s>(
    config: &DemangleConfig,
    s: &'s str,
    cplus_marker: char,
    full_sym: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, which, is_constructor) = if let Some(r) = s.strip_prefix("I") {
        (r, "constructors", true)
    } else if let Some(r) = s.strip_prefix("D") {
        (r, "destructors", false)
    } else if let Some(r) = s.strip_prefix("F") {
        if config.demangle_global_keyed_frames {
            (r, "frames", false)
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
        && is_constructor
        && remaining.starts_with("__Q")
    {
        // !HACK(c++filt): Seems like c++filt has a bug where it won't output
        // !the "global constructors keyed to " prefix for namespaced functions
        return demangled_sym;
    }

    let actual_sym = demangled_sym
        .map(Cow::from)
        .unwrap_or_else(|_| Cow::from(remaining));

    Ok(format!("global {which} keyed to {actual_sym}"))
}

fn demangle_in_function_static_initialized<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if !config.demangle_in_function_static_initialized {
        return Err(DemangleError::InFunctionStaticInitializedDisabled(s));
    }

    let Some((left, right)) = s.c_split2_char('.') else {
        return Err(DemangleError::InFunctionStaticInitializedMissingPair(s));
    };

    let left: usize = left
        .parse()
        .map_err(|_| DemangleError::InFunctionStaticInitializedFirstNotNumber(left))?;
    let right: usize = right
        .parse()
        .map_err(|_| DemangleError::InFunctionStaticInitializedSecondNotNumber(right))?;

    // made up convention
    Ok(format!("in-function-static initialized {left} {right}"))
}
