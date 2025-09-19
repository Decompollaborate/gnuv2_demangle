/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroUsize;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{str_cutter::StrCutter, DemangleConfig, DemangleError};

pub fn demangle<'s>(sym: &'s str, config: &DemangleConfig) -> Result<String, DemangleError<'s>> {
    if !sym.is_ascii() {
        return Err(DemangleError::NonAscii);
    }

    demangle_impl(sym, config, true)
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
    if let Some(s) = s.strip_prefix('t') {
        let (remaining, template, typ) = demangle_template(config, s, &[])?;

        if remaining.is_empty() {
            Ok(format!("{template}::~{typ}(void)"))
        } else {
            Err(DemangleError::TrailingDataOnDestructor(remaining))
        }
    } else if let Some(s) = s.strip_prefix('Q') {
        let (remaining, namespaces, trailing_namespace) = demangle_namespaces(config, s, &[])?;

        if remaining.is_empty() {
            Ok(format!("{namespaces}::~{trailing_namespace}(void)"))
        } else {
            Err(DemangleError::TrailingDataOnDestructor(remaining))
        }
    } else {
        let (remaining, class_name) = demangle_custom_name(s)?;

        if remaining.is_empty() {
            Ok(format!("{class_name}::~{class_name}(void)"))
        } else {
            Err(DemangleError::TrailingDataOnDestructor(remaining))
        }
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
        let (remaining, class_name) = demangle_custom_name(s)?;

        (
            remaining,
            Some(Cow::from(class_name)),
            Cow::from(class_name),
            "",
        )
    } else if let Some(remaining) = s.strip_prefix("tf") {
        return demangle_type_info_function(config, remaining);
    } else if let Some(remaining) = s.strip_prefix("ti") {
        return demangle_type_info_node(config, remaining);
    } else if let Some(remaining) = s.strip_prefix('t') {
        let (remaining, template, typ) = demangle_template(config, remaining, &[])?;

        (remaining, Some(Cow::from(template)), Cow::from(typ), "")
    } else if let Some(q_less) = s.strip_prefix('Q') {
        let (remaining, namespaces, trailing_namespace) = demangle_namespaces(config, q_less, &[])?;

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
                    let (remaining, DemangledArg::Plain(typ)) =
                        demangle_argument(config, cast, None, &[], &[])?
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
            let (remaining, suffix) = demangle_method_qualifier(remaining);

            let (remaining, namespaces) = if let Some(q_less) = remaining.strip_prefix('Q') {
                let (remaining, namespaces, _trailing_namespace) =
                    demangle_namespaces(config, q_less, &[])?;

                (remaining, Cow::from(namespaces))
            } else if let Some(r) = remaining.strip_prefix('t') {
                let (remaining, template, _typ) = demangle_template(config, r, &[])?;

                (remaining, Cow::from(template))
            } else {
                let (remaining, class_name) = demangle_custom_name(remaining)?;

                (remaining, Cow::from(class_name))
            };

            (remaining, Some(namespaces), method_name, suffix)
        }
    };

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, remaining, class_name.as_deref(), &[])?
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
    let argument_list = demangle_argument_list(config, args, None, &[])?;

    Ok(format!("{func_name}({argument_list})"))
}

fn demangle_argument_list<'s>(
    config: &DemangleConfig,
    args: &'s str,
    namespace: Option<&str>,
    template_args: &[String],
) -> Result<String, DemangleError<'s>> {
    let (remaining, argument_list) =
        demangle_argument_list_impl(config, args, namespace, template_args)?;

    if !remaining.is_empty() {
        return Err(DemangleError::TrailingDataAfterArgumentList(remaining));
    }

    Ok(argument_list)
}

fn demangle_argument_list_impl<'s>(
    config: &DemangleConfig,
    mut args: &'s str,
    namespace: Option<&str>,
    template_args: &[String],
) -> Result<(&'s str, String), DemangleError<'s>> {
    let mut arguments = Vec::new();
    let mut trailing_ellipsis = false;

    while !args.is_empty() && !args.starts_with('_') {
        let (remaining, b) = demangle_argument(config, args, namespace, &arguments, template_args)?;

        match b {
            DemangledArg::Plain(s) => arguments.push(s),
            DemangledArg::Repeat { count, index } => {
                for _ in 0..count {
                    let arg = if let Some(namespace) = namespace {
                        if index == 0 {
                            namespace
                        } else {
                            arguments
                                .get(index - 1)
                                .ok_or(DemangleError::InvalidRepeatingArgument(args))?
                        }
                    } else {
                        arguments
                            .get(index)
                            .ok_or(DemangleError::InvalidRepeatingArgument(args))?
                    };
                    // TODO: Look up for a way to avoid cloning, maybe use Cow?
                    arguments.push(arg.to_string());
                }
            }
            DemangledArg::Ellipsis => {
                if !remaining.is_empty() {
                    return Err(DemangleError::TrailingDataAfterEllipsis(remaining));
                }
                trailing_ellipsis = true;
                args = remaining;
                break;
            }
        }

        args = remaining;
    }

    let mut out = arguments.join(", ");
    if trailing_ellipsis {
        // Special case to mimic c++filt, since it doesn't use an space between
        // the comma and the ellipsis.
        out.push_str(",...");
    }
    Ok((args, out))
}

fn demangle_method<'s>(
    config: &DemangleConfig,
    method_name: &'s str,
    class_and_args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, suffix) = demangle_method_qualifier(class_and_args);

    let (remaining, namespace) = if let Some(templated) = remaining.strip_prefix('t') {
        let (remaining, template, _typ) = demangle_template(config, templated, &[])?;

        (remaining, Cow::from(template))
    } else if let Some(q_less) = remaining.strip_prefix('Q') {
        let (remaining, namespaces, _trailing_namespace) =
            demangle_namespaces(config, q_less, &[])?;

        (remaining, Cow::from(namespaces))
    } else if let Some(with_return_type) = remaining.strip_prefix('H') {
        let (remaining, template_args, typ) =
            demangle_template_with_return_type(config, with_return_type)?;

        let (remaining, typ) = if let Some(typ) = typ {
            (remaining, Some(typ))
        } else if let Some(r) = remaining.strip_prefix('t') {
            let (r, template, _typ) = demangle_template(config, r, &[])?;

            (r, Some(Cow::from(template)))
        } else {
            (remaining, None)
        };

        let (remaining, argument_list) =
            demangle_argument_list_impl(config, remaining, typ.as_deref(), &template_args)?;
        let prefix = if let Some(r) = remaining.strip_prefix('_') {
            let (r, DemangledArg::Plain(mut ret_type)) =
                demangle_argument(config, r, typ.as_deref(), &[], &template_args)?
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

        let formated_template_args = if template_args.last().is_some_and(|x| x.ends_with('>')) {
            format!("<{} >", template_args.join(", "))
        } else {
            format!("<{}>", template_args.join(", "))
        };
        return Ok(if let Some(typ) = typ {
            format!("{prefix}{typ}::{method_name}{formated_template_args}({argument_list}){suffix}")
        } else {
            format!("{prefix}{method_name}{formated_template_args}({argument_list}){suffix}")
        });
    } else {
        let (remaining, class_name) = demangle_custom_name(remaining)?;

        (remaining, Cow::from(class_name))
    };

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, remaining, Some(&namespace), &[])?
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
    let (remaining, namespaces, _trailing_namespace) = demangle_namespaces(config, s, &[])?;

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, remaining, Some(&namespaces), &[])?
    };

    let out = format!("{namespaces}::{func_name}({argument_list})");
    Ok(out)
}

fn demangle_type_info_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if let (remaining, DemangledArg::Plain(demangled_type)) =
        demangle_argument(config, s, None, &[], &[])?
    {
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
    if let (remaining, DemangledArg::Plain(demangled_type)) =
        demangle_argument(config, s, None, &[], &[])?
    {
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
            let (r, template, _typ) = demangle_template(config, r, &[])?;

            stuff.push(Cow::from(template));
            r
        } else if let Some(r) = remaining.strip_prefix('Q') {
            let (r, namespaces, _trailing_namespace) = demangle_namespaces(config, r, &[])?;

            stuff.push(Cow::from(namespaces));
            r
        } else {
            let (r, class_name) = demangle_custom_name(remaining)?;

            stuff.push(Cow::from(class_name));
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
        let (r, template, _typ) = demangle_template(config, r, &[])?;

        (r, Cow::from(template))
    } else if let Some(r) = remaining.strip_prefix('Q') {
        let (r, namespaces, _trailing_namespace) = demangle_namespaces(config, r, &[])?;

        (r, Cow::from(namespaces))
    } else {
        let (r, class_name) = demangle_custom_name(remaining)?;

        (r, Cow::from(class_name))
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum DemangledArg {
    Plain(String),
    Repeat { count: usize, index: usize },
    Ellipsis,
}

fn demangle_argument<'s>(
    config: &DemangleConfig,
    full_args: &'s str,
    namespace: Option<&str>,
    parsed_arguments: &[String],
    template_args: &[String],
) -> Result<(&'s str, DemangledArg), DemangleError<'s>> {
    if let Some(repeater) = full_args.strip_prefix('N') {
        let remaining = repeater;
        let (remaining, count) = parse_number_maybe_multi_digit(remaining)
            .ok_or(DemangleError::InvalidRepeatingArgument(full_args))?;
        if count == 0 {
            return Err(DemangleError::InvalidRepeatingArgument(full_args));
        }
        let (remaining, index) = parse_number_maybe_multi_digit(remaining)
            .ok_or(DemangleError::InvalidRepeatingArgument(full_args))?;
        return Ok((remaining, DemangledArg::Repeat { count, index }));
    } else if let Some(remaining) = full_args.strip_prefix('e') {
        return Ok((remaining, DemangledArg::Ellipsis));
    } else if let Some(remaining) = full_args.strip_prefix('t') {
        let (remaining, template, _typ) = demangle_template(config, remaining, template_args)?;

        return Ok((remaining, DemangledArg::Plain(template)));
    }

    let mut out = String::new();
    let mut post = String::new();
    let mut args = full_args;

    // Qualifiers
    while !args.is_empty() {
        let c = args
            .chars()
            .next()
            .ok_or(DemangleError::RanOutOfArguments)?;

        match c {
            'P' => post.insert(0, '*'),
            'R' => post.insert(0, '&'),
            'C' => post.insert_str(0, "const "),
            'S' => out.push_str("signed "),
            'U' => out.push_str("unsigned "),
            _ => break,
        }

        args = &args[1..];
    }

    if args.starts_with('A') {
        post.insert(0, '(');
        post.push(')');

        while let Some(remaining) = args.strip_prefix('A') {
            let Some((remaining, array_length)) = parse_number(remaining) else {
                return Err(DemangleError::InvalidArraySize(remaining));
            };
            let Some(remaining) = remaining.strip_prefix('_') else {
                return Err(DemangleError::MalformedArrayArgumment(remaining));
            };

            let array_length = if config.fix_array_length_arg {
                array_length + 1
            } else {
                array_length
            };

            post.push_str(&format!("[{array_length}]"));

            args = remaining;
        }

        // Do qualifiers again for the type
        while !args.is_empty() {
            let c = args
                .chars()
                .next()
                .ok_or(DemangleError::RanOutOfArguments)?;

            match c {
                'P' => post.insert(0, '*'),
                'R' => post.insert(0, '&'),
                'C' => post.insert_str(0, "const "),
                'S' => out.push_str("signed "),
                'U' => out.push_str("unsigned "),
                _ => break,
            }

            args = &args[1..];
        }
    }

    // 'G' is used for classes, structs and unions, so we must make sure we
    // don't parse a primitive type next, otherwise this is not properly
    // mangled.
    let must_be_class_like = if let Some(a) = args.strip_prefix('G') {
        args = a;
        true
    } else {
        false
    };

    let c = args
        .chars()
        .next()
        .ok_or(DemangleError::RanOutOfArguments)?;
    let mut is_class_like = false;
    // Plain types
    match c {
        'c' => out.push_str("char"),
        's' => out.push_str("short"),
        'i' => out.push_str("int"),
        'l' => out.push_str("long"),
        'x' => out.push_str("long long"),
        'f' => out.push_str("float"),
        'd' => out.push_str("double"),
        'r' => out.push_str("long double"),
        'b' => out.push_str("bool"),
        'w' => out.push_str("wchar_t"),
        'v' => out.push_str("void"),
        '1'..='9' => {
            let (remaining, class_name) = demangle_custom_name(args)?;
            args = remaining;
            is_class_like = true;
            out.push_str(class_name);
        }
        'Q' => {
            let (remaining, namespaces, _trailing_namespace) =
                demangle_namespaces(config, &args[1..], template_args)?;
            args = remaining;
            is_class_like = true;
            out.push_str(&namespaces);
        }
        'T' => {
            // Remembered type / look back
            let (remaining, lookback) = parse_number_maybe_multi_digit(&args[1..])
                .ok_or(DemangleError::InvalidLookbackCount(args))?;

            let referenced_arg = if let Some(namespace) = namespace {
                if lookback == 0 {
                    namespace
                } else {
                    parsed_arguments
                        .get(lookback - 1)
                        .ok_or(DemangleError::LookbackCountTooBig(args, lookback))?
                }
            } else {
                parsed_arguments
                    .get(lookback)
                    .ok_or(DemangleError::LookbackCountTooBig(args, lookback))?
            };

            args = remaining;

            // Not really, since lookback could reference anything...
            is_class_like = true;

            out.push_str(referenced_arg);
        }
        // TODO :is this dead code?
        't' => {
            // templates
            let (remaining, template, _typ) = demangle_template(config, &args[1..], template_args)?;

            args = remaining;

            is_class_like = true;
            out.push_str(&template);
        }
        'F' => {
            // Function pointer/reference
            args = &args[1..];

            let mut subargs = Vec::new();
            let namespace = None;
            let mut trailing_ellipsis = false;
            while !args.starts_with('_') {
                let (r, arg) = demangle_argument(config, args, namespace, &subargs, &[])?;

                match arg {
                    DemangledArg::Plain(s) => subargs.push(s),
                    DemangledArg::Repeat { count, index } => {
                        for _ in 0..count {
                            let arg = if let Some(namespace) = namespace {
                                if index == 0 {
                                    namespace
                                } else {
                                    subargs
                                        .get(index - 1)
                                        .ok_or(DemangleError::InvalidRepeatingArgument(args))?
                                }
                            } else {
                                subargs
                                    .get(index)
                                    .ok_or(DemangleError::InvalidRepeatingArgument(args))?
                            };
                            // TODO: Look up for a way to avoid cloning, maybe use Cow?
                            subargs.push(arg.to_string());
                        }
                    }
                    DemangledArg::Ellipsis => {
                        trailing_ellipsis = true;
                    }
                }
                args = r;
            }

            let Some(r) = args.strip_prefix('_') else {
                return Err(DemangleError::MissingReturnTypeForFunctionPointer(args));
            };
            args = r;

            let (r, DemangledArg::Plain(ret)) =
                demangle_argument(config, args, namespace, &subargs, &[])?
            else {
                return Err(DemangleError::InvalidReturnTypeForFunctionPointer(args));
            };
            args = r;

            let mut argument_list = subargs.join(", ");
            if trailing_ellipsis {
                argument_list.push_str(",...");
            }
            return Ok((
                args,
                DemangledArg::Plain(format!(
                    "{}{}({})({})",
                    ret,
                    if ret.ends_with(['*', '&']) { "" } else { " " },
                    post.trim_matches(' '),
                    argument_list
                )),
            ));
        }
        'X' => {
            args = &args[1..];
            let (r, index) = if let Some(r) = args.strip_prefix('_') {
                parse_number_maybe_multi_digit(r)
            } else {
                parse_digit(args)
            }
            .ok_or(DemangleError::InvalidValueForIndexOnXArgument(args))?;
            let Some((r, number1)) = parse_digit(r) else {
                return Err(DemangleError::InvalidValueForNumber1OnXArgument(r));
            };
            // TODO: what is this number?
            if number1 != 1 && number1 != 0 {
                return Err(DemangleError::InvalidNumber1OnXArgument(r, number1));
            }

            let Some(t) = template_args.get(index) else {
                return Err(DemangleError::IndexTooBigForXArgument(r, index));
            };
            out.push_str(t);

            args = r;
            is_class_like = true;
        }
        _ => {
            return Err(DemangleError::UnknownType(c, args));
        }
    }

    if must_be_class_like && !is_class_like {
        return Err(DemangleError::PrimitiveInsteadOfClass(full_args));
    }

    if !is_class_like {
        args = &args[1..];
    }

    let out = if !post.is_empty() {
        out + " " + post.trim_matches(' ')
    } else {
        out
    };

    Ok((args, DemangledArg::Plain(out)))
}

// 'Q' must be stripped already
fn demangle_namespaces<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &[String],
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let (remaining, namespace_count) = if let Some(r) = s.strip_prefix('_') {
        // More than a single digit of namespaces
        parse_number(r).and_then(|(r, l)| r.strip_prefix('_').map(|new_r| (new_r, l)))
    } else {
        parse_digit(s)
    }
    .ok_or(DemangleError::InvalidNamespaceCount(s))?;
    let namespace_count =
        NonZeroUsize::new(namespace_count).ok_or(DemangleError::InvalidNamespaceCount(s))?;

    demangle_namespaces_impl(config, remaining, namespace_count, template_args)
}

fn demangle_namespaces_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespace_count: NonZeroUsize,
    template_args: &[String],
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let mut namespaces = String::new();
    let mut remaining = s;
    let mut trailing_type = "";

    for _ in 0..namespace_count.get() {
        if !namespaces.is_empty() {
            namespaces.push_str("::");
        }

        let (r, n) = if let Some(temp) = remaining.strip_prefix('t') {
            let (r, template, typ) = demangle_template(config, temp, template_args)?;
            trailing_type = typ;
            (r, Cow::from(template))
        } else {
            let (r, ns) = demangle_custom_name(remaining)?;
            trailing_type = ns;
            (r, Cow::from(ns))
        };
        remaining = r;
        namespaces.push_str(&n);
    }

    Ok((remaining, namespaces, trailing_type))
}

fn demangle_template<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &[String],
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let (remaining, class_name) = demangle_custom_name(s)?;
    let Some((remaining, digit)) = parse_digit(remaining) else {
        return Err(DemangleError::InvalidTemplateCount(remaining));
    };

    let (remaining, types) = demangle_template_types_impl(config, remaining, digit, template_args)?;

    let template = if types.last().is_some_and(|x| x.ends_with('>')) {
        format!("{}<{} >", class_name, types.join(", "))
    } else {
        format!("{}<{}>", class_name, types.join(", "))
    };
    Ok((remaining, template, class_name))
}

// TODO: fix this
#[allow(clippy::type_complexity)]
fn demangle_template_with_return_type<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<(&'s str, Vec<String>, Option<Cow<'s, str>>), DemangleError<'s>> {
    let remaining = s;
    let Some((remaining, digit)) = parse_digit(remaining) else {
        return Err(DemangleError::InvalidTemplateReturnCount(remaining));
    };

    let (remaining, types) = demangle_template_types_impl(config, remaining, digit, &[])?;

    let Some(remaining) = remaining.strip_prefix('_') else {
        return Err(DemangleError::MalformedTemplateWithReturnType(remaining));
    };
    let (remaining, namespaces) = if let Some(q_less) = remaining.strip_prefix('Q') {
        let (r, namespaces, _trailing_namespace) = demangle_namespaces(config, q_less, &[])?;

        (r, Some(Cow::from(namespaces)))
    } else if remaining.starts_with(|c| matches!(c, '1'..='9')) {
        let (r, namespace) = demangle_custom_name(remaining)?;
        (r, Some(Cow::from(namespace)))
    } else {
        (remaining, None)
    };

    Ok((remaining, types, namespaces))
}

fn demangle_template_types_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    count: usize, // TODO: NonZero
    template_args: &[String],
) -> Result<(&'s str, Vec<String>), DemangleError<'s>> {
    let mut remaining = s;

    let namespace = None;
    let mut types = Vec::new();

    for _ in 0..count {
        let r = if let Some(r) = remaining.strip_prefix('Z') {
            let (r, arg) = demangle_argument(config, r, namespace, &types, template_args)?;

            match arg {
                DemangledArg::Plain(s) => types.push(s),
                DemangledArg::Repeat { count, index } => {
                    for _ in 0..count {
                        let arg = if let Some(namespace) = namespace {
                            if index == 0 {
                                namespace
                            } else {
                                types
                                    .get(index - 1)
                                    .ok_or(DemangleError::InvalidRepeatingArgument(s))?
                            }
                        } else {
                            types
                                .get(index)
                                .ok_or(DemangleError::InvalidRepeatingArgument(s))?
                        };
                        // TODO: Look up for a way to avoid cloning, maybe use Cow?
                        types.push(arg.to_string());
                    }
                }
                DemangledArg::Ellipsis => {
                    if !r.is_empty() {
                        return Err(DemangleError::TrailingDataAfterEllipsis(r));
                    }
                    types.push("...".to_string());
                    break;
                }
            }

            r
        } else {
            let mut r = remaining;
            let mut is_pointer = false;
            let mut is_reference = false;

            // Skip over any known qualifier
            while !r.is_empty() {
                let c = r.chars().next().ok_or(DemangleError::RanOutOfArguments)?;

                match c {
                    // '*'
                    'P' => is_pointer = true,
                    // '&'
                    'R' => is_reference = true,
                    // "const"
                    'C' => {}
                    // "signed" | "unsigned"
                    'S' | 'U' => {}
                    _ => break,
                }

                r = &r[1..];
            }

            if is_pointer || is_reference {
                let (aux, DemangledArg::Plain(_arg)) =
                    demangle_argument(config, r, None, &[], &[])?
                else {
                    return Err(DemangleError::InvalidTemplatedPointerReferenceValue(r));
                };
                let (aux, symbol) = demangle_custom_name(aux)?;
                types.push(format!("{}{}", if is_pointer { "&" } else { "" }, symbol));
                aux
            } else {
                let c = r.chars().next().ok_or(DemangleError::RanOutOfArguments)?;
                r = &r[1..];

                match c {
                    // "char" | "wchar_t"
                    'c' | 'w' => {
                        let (r, number) = parse_number(r)
                            .ok_or(DemangleError::InvalidTemplatedNumberForCharacterValue(r))?;
                        let demangled_char = char::from_u32(number.try_into().map_err(|_| {
                            DemangleError::InvalidTemplatedCharacterValue(r, number)
                        })?)
                        .ok_or(DemangleError::InvalidTemplatedCharacterValue(r, number))?;
                        types.push(format!("'{demangled_char}'"));
                        r
                    }
                    // "short" | "int" | "long" | "long long"
                    's' | 'i' | 'l' | 'x' => {
                        let (r, negative) = if let Some(r) = r.strip_prefix('m') {
                            (r, true)
                        } else {
                            (r, false)
                        };
                        let (r, number) = parse_number(r)
                            .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?;
                        types.push(format!("{}{}", if negative { "-" } else { "" }, number));
                        r
                    }
                    // 'f' => {}, // "float"
                    // 'd' => {}, // "double"
                    // 'r' => {}, // "long double"
                    // "bool"
                    'b' => match r.chars().next() {
                        Some('1') => {
                            types.push("true".to_string());
                            &r[1..]
                        }
                        Some('0') => {
                            types.push("false".to_string());
                            &r[1..]
                        }
                        _ => return Err(DemangleError::InvalidTemplatedBoolean(r)),
                    },
                    _ => return Err(DemangleError::InvalidTypeValueForTemplated(c, r)),
                }
            }
        };

        remaining = r;
    }

    Ok((remaining, types))
}

fn demangle_custom_name<'s>(s: &'s str) -> Result<(&'s str, &'s str), DemangleError<'s>> {
    let (remaining, length) = parse_number(s).ok_or(DemangleError::InvalidCustomNameCount(s))?;

    if remaining.len() < length {
        Err(DemangleError::RanOutOfCharactersForCustomName(s))
    } else {
        Ok((&remaining[length..], &remaining[..length]))
    }
}

fn demangle_method_qualifier(s: &str) -> (&str, &str) {
    if let Some(remaining) = s.strip_prefix('C') {
        (remaining, " const")
    } else {
        (s, "")
    }
}

fn parse_number(s: &str) -> Option<(&str, usize)> {
    let ret = if let Some(index) = s.find(|c: char| !c.is_ascii_digit()) {
        (&s[index..], s[..index].parse().ok()?)
    } else {
        ("", s.parse().ok()?)
    };

    Some(ret)
}

fn parse_digit(s: &str) -> Option<(&str, usize)> {
    let c = s.chars().next()?;
    if c.is_ascii_digit() {
        let digit = (c as usize).wrapping_sub('0' as usize);

        Some((&s[1..], digit))
    } else {
        None
    }
}

/// Parse either a single digit followed by nondigits or a multidigit followed
/// by an underscore.
fn parse_number_maybe_multi_digit(s: &str) -> Option<(&str, usize)> {
    if s.is_empty() {
        None
    } else if s.len() == 1 {
        // Single digit should be fine to just parse
        Some(("", s.parse().ok()?))
    } else if let Some(index) = s.find(|c: char| !c.is_ascii_digit()) {
        if index == 0 {
            None
        } else if s[index..].starts_with('_') {
            // Number can be followed by an underscore only if it is a
            // multidigit value
            if index > 1 {
                Some((&s[index + 1..], s[..index].parse().ok()?))
            } else {
                None
            }
        } else {
            // Only consume a single digit
            Some((&s[1..], s[..1].parse().ok()?))
        }
    } else {
        // Only consume a single digit
        Some((&s[1..], s[..1].parse().ok()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_maybe_multi_digit() {
        assert_eq!(parse_number_maybe_multi_digit("1junk"), Some(("junk", 1)),);
        assert_eq!(
            parse_number_maybe_multi_digit("12_junk"),
            Some(("junk", 12)),
        );
        assert_eq!(parse_number_maybe_multi_digit("54junk"), Some(("4junk", 5)),);
        assert_eq!(parse_number_maybe_multi_digit("2"), Some(("", 2)),);
        assert_eq!(parse_number_maybe_multi_digit("32"), Some(("2", 3)),);
        assert_eq!(parse_number_maybe_multi_digit("1_junk"), None,);
    }
}
