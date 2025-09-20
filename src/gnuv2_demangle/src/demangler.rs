/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::{DemangleConfig, DemangleError};

use crate::{
    dem::{demangle_custom_name, demangle_method_qualifier},
    dem_arg::{demangle_argument, DemangledArg},
    dem_arg_list::{demangle_argument_list, demangle_argument_list_impl, ArgVec},
    dem_namespace::demangle_namespaces,
    dem_template::{demangle_template, demangle_template_with_return_type},
    remainer::Remaining,
    str_cutter::StrCutter,
};

pub fn demangle<'s>(sym: &'s str, config: &DemangleConfig) -> Result<String, DemangleError<'s>> {
    if !sym.is_ascii() {
        Err(DemangleError::NonAscii)
    } else {
        demangle_impl(sym, config, true)
    }
}

fn demangle_impl<'s>(
    sym: &'s str,
    config: &DemangleConfig,
    allow_global_sym_keyed: bool,
) -> Result<String, DemangleError<'s>> {
    if let Some(s) = sym.strip_prefix("_$_") {
        demangle_destructor(config, s)
    } else if let Some(s) = sym.strip_prefix("__") {
        demangle_special(config, s, sym)
    } else if let Some(s) = sym.c_cond_and_strip_prefix(allow_global_sym_keyed, "_GLOBAL_$") {
        demangle_global_sym_keyed(config, s, sym)
    } else if let Some((func_name, args)) = sym.c_split2("__F") {
        demangle_free_function(config, func_name, args)
    } else if let Some((method_name, class_and_args)) =
        sym.c_split2_r_starts_with("__", |c| matches!(c, '1'..='9' | 'C' | 't' | 'H'))
    {
        demangle_method(config, method_name, class_and_args)
    } else if let Some((func_name, s)) = sym.c_split2("__Q") {
        demangle_namespaced_function(config, func_name, s)
    } else if let Some(sym) = sym.strip_prefix("_vt") {
        demangle_virtual_table(config, sym)
    } else if let Some((s, name)) = sym.c_split2("$") {
        demangle_namespaced_global(config, s, name)
    } else {
        Err(DemangleError::NotMangled)
    }
}

fn demangle_destructor<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (r, namespace, typ) = if let Some(s) = s.strip_prefix('t') {
        let (r, template, typ) = demangle_template(config, s, &ArgVec::new(config, None))?;
        (r, Cow::from(template), Cow::from(typ))
    } else if let Some(s) = s.strip_prefix('Q') {
        let (r, namespaces, trailing_namespace) =
            demangle_namespaces(config, s, &ArgVec::new(config, None))?;
        (r, Cow::from(namespaces), Cow::from(trailing_namespace))
    } else {
        let Remaining { r, d: class_name } =
            demangle_custom_name(s, DemangleError::InvalidClassNameOnDestructor)?;
        (r, Cow::from(class_name), Cow::from(class_name))
    };

    if r.is_empty() {
        Ok(format!("{namespace}::~{typ}(void)"))
    } else {
        Err(DemangleError::TrailingDataOnDestructor(r))
    }
}

fn demangle_special<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_sym: &'s str,
) -> Result<String, DemangleError<'s>> {
    let c = s
        .chars()
        .next()
        .ok_or(DemangleError::RanOutWhileDemanglingSpecial)?;

    let (remaining, class_name, method_name, suffix) = if matches!(c, '1'..='9') {
        // class constructor
        let Remaining { r, d: class_name } =
            demangle_custom_name(s, DemangleError::InvalidClassNameOnConstructor)?;

        (r, Some(Cow::from(class_name)), Cow::from(class_name), "")
    } else if let Some(remaining) = s.strip_prefix("tf") {
        return demangle_type_info_function(config, remaining);
    } else if let Some(remaining) = s.strip_prefix("ti") {
        return demangle_type_info_node(config, remaining);
    } else if let Some(remaining) = s.strip_prefix('t') {
        let (remaining, template, typ) =
            demangle_template(config, remaining, &ArgVec::new(config, None))?;

        (remaining, Some(Cow::from(template)), Cow::from(typ), "")
    } else if let Some(q_less) = s.strip_prefix('Q') {
        let (remaining, namespaces, trailing_namespace) =
            demangle_namespaces(config, q_less, &ArgVec::new(config, None))?;

        (
            remaining,
            Some(Cow::from(namespaces)),
            Cow::from(trailing_namespace),
            "",
        )
    } else {
        let end_index = s.find("__").ok_or(DemangleError::InvalidSpecialMethod(s))?;
        let op = &s[..end_index];

        let remaining = &s[end_index + 2..];

        let method_name = match op {
            "nw" => Cow::from("operator new"),
            "dl" => Cow::from("operator delete"),
            "vn" => Cow::from("operator new []"),
            "eq" => Cow::from("operator=="),
            "ne" => Cow::from("operator!="),
            "as" => Cow::from("operator="),
            "vc" => Cow::from("operator[]"),
            "ad" => Cow::from("operator&"),
            "nt" => Cow::from("operator!"),
            "ls" => Cow::from("operator<<"),
            "rs" => Cow::from("operator>>"),
            "er" => Cow::from("operator^"),
            "lt" => Cow::from("operator<"),
            "aml" => Cow::from("operator*="),
            "apl" => Cow::from("operator+="),
            _ => {
                if let Some(cast) = op.strip_prefix("op") {
                    let (remaining, DemangledArg::Plain(typ)) = demangle_argument(
                        config,
                        cast,
                        &ArgVec::new(config, None),
                        &ArgVec::new(config, None),
                    )?
                    else {
                        return Err(DemangleError::UnrecognizedSpecialMethod(op));
                    };
                    if !remaining.is_empty() {
                        return Err(DemangleError::MalformedCastOperatorOverload(remaining));
                    }

                    Cow::from(format!("operator {typ}"))
                } else {
                    return {
                        // This may be a plain function that got confused with a
                        // special symbol, so try to decode as a function instead.
                        if let Some((func_name, args)) = full_sym.c_split2("__F") {
                            demangle_free_function(config, func_name, args)
                        } else if let Some((incomplete_method_name, class_and_args)) = s
                            .c_split2_r_starts_with("__", |c| {
                                matches!(c, '1'..='9' | 'C' | 't' | 'H')
                            })
                        {
                            // split `s` instead of `full_sym` to skip over the
                            // first `__`,
                            // if that check passes, then recover the actual
                            // method name, including the initial `__`, by
                            // using the length of the `incomplete_method_name`
                            // to slice the `full_sym`.

                            let method_name = &full_sym[..incomplete_method_name.len() + 2];
                            demangle_method(config, method_name, class_and_args)
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

            let (remaining, namespaces) = if let Some(q_less) = remaining.strip_prefix('Q') {
                let (remaining, namespaces, _trailing_namespace) =
                    demangle_namespaces(config, q_less, &ArgVec::new(config, None))?;

                (remaining, Cow::from(namespaces))
            } else if let Some(r) = remaining.strip_prefix('t') {
                let (remaining, template, _typ) =
                    demangle_template(config, r, &ArgVec::new(config, None))?;

                (remaining, Cow::from(template))
            } else {
                let Remaining { r, d: class_name } =
                    demangle_custom_name(remaining, DemangleError::InvalidClassNameOnOperator)?
                        .d_as_cow();

                (r, class_name)
            };

            (remaining, Some(namespaces), method_name, suffix)
        }
    };

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(
            config,
            remaining,
            class_name.as_deref(),
            &ArgVec::new(config, None),
        )?
    };

    let out = if let Some(class_name) = class_name {
        format!("{class_name}::{method_name}({argument_list}){suffix}")
    } else {
        format!("{method_name}({argument_list}){suffix}")
    };
    Ok(out)
}

fn demangle_free_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let argument_list = demangle_argument_list(config, args, None, &ArgVec::new(config, None))?;

    Ok(format!("{func_name}({argument_list})"))
}

fn demangle_method<'s>(
    config: &DemangleConfig,
    method_name: &'s str,
    class_and_args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let Remaining {
        r: remaining,
        d: suffix,
    } = demangle_method_qualifier(class_and_args);

    let (remaining, namespace) = if let Some(templated) = remaining.strip_prefix('t') {
        let (remaining, template, _typ) =
            demangle_template(config, templated, &ArgVec::new(config, None))?;

        (remaining, Cow::from(template))
    } else if let Some(q_less) = remaining.strip_prefix('Q') {
        let (remaining, namespaces, _trailing_namespace) =
            demangle_namespaces(config, q_less, &ArgVec::new(config, None))?;

        (remaining, Cow::from(namespaces))
    } else if let Some(with_return_type) = remaining.strip_prefix('H') {
        let (remaining, template_args, typ) =
            demangle_template_with_return_type(config, with_return_type)?;

        let (remaining, typ) = if let Some(typ) = typ {
            (remaining, Some(typ))
        } else if let Some(r) = remaining.strip_prefix('t') {
            let (r, template, _typ) = demangle_template(config, r, &ArgVec::new(config, None))?;

            (r, Some(Cow::from(template)))
        } else {
            (remaining, None)
        };

        let (remaining, argument_list) =
            demangle_argument_list_impl(config, remaining, typ.as_deref(), &template_args, false)?;
        let prefix = if let Some(r) = remaining.strip_prefix('_') {
            let (r, DemangledArg::Plain(mut ret_type)) = demangle_argument(
                config,
                r,
                &ArgVec::new(config, typ.as_deref()),
                &template_args,
            )?
            else {
                return Err(DemangleError::MalformedTemplateWithReturnTypeMissingReturnType(r));
            };
            if !r.is_empty() {
                return Err(
                    DemangleError::TrailingDataAfterReturnTypeOfMalformedTemplateWithReturnType(r),
                );
            }
            ret_type.push(' ');
            ret_type
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
        return Ok(if let Some(typ) = typ {
            format!("{prefix}{typ}::{method_name}{formated_template_args}({argument_list}){suffix}")
        } else {
            format!("{prefix}{method_name}{formated_template_args}({argument_list}){suffix}")
        });
    } else {
        let Remaining { r, d: class_name } =
            demangle_custom_name(remaining, DemangleError::InvalidClassNameOnMethod)?.d_as_cow();

        (r, class_name)
    };

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(
            config,
            remaining,
            Some(&namespace),
            &ArgVec::new(config, None),
        )?
    };

    Ok(format!(
        "{namespace}::{method_name}({argument_list}){suffix}"
    ))
}

fn demangle_namespaced_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, namespaces, _trailing_namespace) =
        demangle_namespaces(config, s, &ArgVec::new(config, None))?;

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(
            config,
            remaining,
            Some(&namespaces),
            &ArgVec::new(config, None),
        )?
    };

    let out = format!("{namespaces}::{func_name}({argument_list})");
    Ok(out)
}

fn demangle_type_info_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if let (remaining, DemangledArg::Plain(demangled_type)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &ArgVec::new(config, None),
    )? {
        if remaining.is_empty() {
            Ok(format!("{demangled_type} type_info function"))
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
    if let (remaining, DemangledArg::Plain(demangled_type)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &ArgVec::new(config, None),
    )? {
        if remaining.is_empty() {
            Ok(format!("{demangled_type} type_info node"))
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
) -> Result<String, DemangleError<'s>> {
    let mut remaining = s;
    let mut stuff = Vec::new();

    while !remaining.is_empty() {
        remaining = remaining
            .strip_prefix('$')
            .ok_or(DemangleError::VTableMissingDollarSeparator(remaining))?;

        remaining = if let Some(r) = remaining.strip_prefix('t') {
            let (r, template, _typ) = demangle_template(config, r, &ArgVec::new(config, None))?;

            stuff.push(Cow::from(template));
            r
        } else if let Some(r) = remaining.strip_prefix('Q') {
            let (r, namespaces, _trailing_namespace) =
                demangle_namespaces(config, r, &ArgVec::new(config, None))?;

            stuff.push(Cow::from(namespaces));
            r
        } else {
            let Remaining { r, d: class_name } =
                demangle_custom_name(remaining, DemangleError::InvalidClassNameOnVirtualTable)?
                    .d_as_cow();

            stuff.push(class_name);
            r
        };
    }

    Ok(format!("{} virtual table", stuff.join("::")))
}

fn demangle_namespaced_global<'s>(
    config: &DemangleConfig,
    s: &'s str,
    name: &'s str,
) -> Result<String, DemangleError<'s>> {
    let Some(remaining) = s.strip_prefix('_') else {
        return Err(DemangleError::InvalidNamespacedGlobal(s, name));
    };

    let (r, space) = if let Some(r) = remaining.strip_prefix('t') {
        let (r, template, _typ) = demangle_template(config, r, &ArgVec::new(config, None))?;

        (r, Cow::from(template))
    } else if let Some(r) = remaining.strip_prefix('Q') {
        let (r, namespaces, _trailing_namespace) =
            demangle_namespaces(config, r, &ArgVec::new(config, None))?;

        (r, Cow::from(namespaces))
    } else {
        let Remaining { r, d: class_name } =
            demangle_custom_name(remaining, DemangleError::InvalidNamespaceOnNamespacedGlobal)?
                .d_as_cow();

        (r, class_name)
    };

    if !r.is_empty() {
        return Err(DemangleError::TrailingDataOnNamespacedGlobal(r));
    }

    Ok(format!("{space}::{name}"))
}

fn demangle_global_sym_keyed<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_sym: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, which, is_constructor) = if let Some(r) = s.strip_prefix("I$") {
        (r, "constructors", true)
    } else if let Some(r) = s.strip_prefix("D$") {
        (r, "destructors", false)
    } else if let Some(r) = s.strip_prefix("F$") {
        if config.demangle_global_keyed_frames {
            (r, "frames", false)
        } else {
            // !HACK(c++filt): c++filt does not recognize `_GLOBAL_$F$`, so it
            // !tries to demangle it as anything else.
            return demangle_impl(full_sym, config, false);
        }
    } else {
        return Err(DemangleError::InvalidGlobalSymKeyed(s));
    };

    let demangled_sym = demangle_impl(remaining, config, false);
    if config.preserve_namespaced_global_constructor_bug
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
