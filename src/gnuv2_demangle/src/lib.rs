/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
#![no_std]
*/

extern crate alloc;

use core::num::NonZeroUsize;

use alloc::borrow::Cow;
use alloc::string::String;

mod demangle_error;

pub use demangle_error::DemangleError;

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
}

impl DemangleConfig {
    /// The default config mimics the default (rather questionable) c++filt's
    /// behavior, including what may be considered c++filt bugs.
    pub fn new() -> Self {
        Self {
            preserve_namespaced_global_constructor_bug: true,
            fix_array_length_arg: false,
        }
    }

    /// Avoid using any option that mimics c++filt faults.
    pub fn new_no_cfilt_mimics() -> Self {
        Self {
            preserve_namespaced_global_constructor_bug: false,
            fix_array_length_arg: true,
        }
    }
}

impl Default for DemangleConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn demangle<'s>(sym: &'s str, config: &DemangleConfig) -> Result<String, DemangleError<'s>> {
    demangle_impl(sym, config, true)
}

fn demangle_impl<'s>(
    sym: &'s str,
    config: &DemangleConfig,
    allow_global_sym_keyed: bool,
) -> Result<String, DemangleError<'s>> {
    if !sym.is_ascii() {
        return Err(DemangleError::NonAscii);
    }

    if let Some(s) = sym.strip_prefix("_$_") {
        demangle_destructor(config, s)
    } else if let Some(s) = sym.strip_prefix("__") {
        demangle_special(config, s, sym)
    } else if let Some(s) = sym.strip_prefix("_GLOBAL_$") {
        if allow_global_sym_keyed {
            demangle_global_sym_keyed(config, s)
        } else {
            Err(DemangleError::NotMangled)
        }
    } else if let Some((func_name, args)) = str_split_2(sym, "__F") {
        demangle_free_function(config, func_name, args)
    } else if let Some((method_name, class_and_args)) =
        str_split_2_second_starts_with(sym, "__", |c| matches!(c, '1'..='9' | 'C' | 't'))
    {
        demangle_method(config, method_name, class_and_args)
    } else if let Some((func_name, s)) = str_split_2(sym, "__Q") {
        demangle_namespaced_function(config, func_name, s)
    } else if let Some(sym) = sym.strip_prefix("_vt") {
        demangle_virtual_table(config, sym)
    } else if let Some((s, name)) = str_split_2(sym, "$") {
        demangle_namespaced_global(config, s, name)
    } else {
        Err(DemangleError::NotMangled)
    }
}

fn str_split_2<'a>(s: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    let mut iter = s.splitn(2, pat);

    if let (Some(l), Some(r)) = (iter.next(), iter.next()) {
        if l.is_empty() {
            None
        } else {
            Some((l, r))
        }
    } else {
        None
    }
}

fn str_split_2_second_starts_with<'a, F>(
    s: &'a str,
    pat: &str,
    second_start: F,
) -> Option<(&'a str, &'a str)>
where
    F: Fn(char) -> bool,
{
    let mut iter = s.splitn(2, pat);

    if let (Some(l), Some(r)) = (iter.next(), iter.next()) {
        if l.is_empty() {
            None
        } else if r.starts_with(second_start) {
            Some((l, r))
        } else {
            None
        }
    } else {
        None
    }
}

fn demangle_destructor<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if let Some(s) = s.strip_prefix('t') {
        let (remaining, template, typ) = demangle_template(config, s)?;

        if remaining.is_empty() {
            Ok(format!("{template}::~{typ}(void)"))
        } else {
            Err(DemangleError::TrailingDataOnDestructor(remaining))
        }
    } else if let Some(s) = s.strip_prefix('Q') {
        let (remaining, namespaces, trailing_namespace) = demangle_namespaces(config, s)?;

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

    let (s, class_name, method_name, suffix) = if matches!(c, '1'..='9') {
        // class constructor
        let (s, class_name) = demangle_custom_name(s)?;

        (s, Some(class_name), Cow::from(class_name), "")
    } else if let Some(s) = s.strip_prefix("tf") {
        return demangle_type_info_function(config, s);
    } else if let Some(s) = s.strip_prefix("ti") {
        return demangle_type_info_node(config, s);
    } else if let Some(s) = s.strip_prefix('t') {
        let (remaining, template, typ) = demangle_template(config, s)?;

        let argument_list = if remaining.is_empty() {
            "void"
        } else {
            &demangle_argument_list(config, remaining, Some(&template))?
        };

        let out = format!("{template}::{typ}({argument_list})");
        return Ok(out);
    } else if let Some(q_less) = s.strip_prefix('Q') {
        // This block is silly, it may be worth to refactor it
        let (remaining, namespaces, trailing_namespace) = demangle_namespaces(config, q_less)?;

        let argument_list = if remaining.is_empty() {
            "void"
        } else {
            &demangle_argument_list(config, remaining, Some(&namespaces))?
        };

        let out = format!("{namespaces}::{trailing_namespace}({argument_list})");
        return Ok(out);
    } else {
        let end_index = s.find("__").ok_or(DemangleError::InvalidSpecialMethod(s))?;
        let op = &s[..end_index];

        let s = &s[end_index + 2..];

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
                        demangle_argument(config, cast, None, &[])?
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
                        if let Some((func_name, args)) = str_split_2(full_sym, "__F") {
                            demangle_free_function(config, func_name, args)
                        } else {
                            Err(DemangleError::UnrecognizedSpecialMethod(op))
                        }
                    };
                }
            }
        };

        if let Some(s) = s.strip_prefix('F') {
            (s, None, method_name, "")
        } else if let Some(q_less) = s.strip_prefix('Q') {
            let (remaining, namespaces, _trailing_namespace) = demangle_namespaces(config, q_less)?;

            let argument_list = if remaining.is_empty() {
                "void"
            } else {
                &demangle_argument_list(config, remaining, Some(&namespaces))?
            };

            let out = format!("{namespaces}::{method_name}({argument_list})");
            return Ok(out);
        } else {
            let (s, suffix) = demangle_method_qualifier(s);
            let (s, class_name) = demangle_custom_name(s)?;

            (s, Some(class_name), method_name, suffix)
        }
    };

    let argument_list = if s.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, s, class_name)?
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
    let argument_list = demangle_argument_list(config, args, None)?;

    Ok(format!("{func_name}({argument_list})"))
}

fn demangle_argument_list<'s>(
    config: &DemangleConfig,
    mut args: &'s str,
    namespace: Option<&str>,
) -> Result<String, DemangleError<'s>> {
    let mut arguments = Vec::new();
    let mut trailing_ellipsis = false;

    while !args.is_empty() {
        let (remaining, b) = demangle_argument(config, args, namespace, &arguments)?;

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
    Ok(out)
}

fn demangle_method<'s>(
    config: &DemangleConfig,
    method_name: &'s str,
    class_and_args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, suffix) = demangle_method_qualifier(class_and_args);

    if let Some(templated) = remaining.strip_prefix('t') {
        let (remaining, template, _typ) = demangle_template(config, templated)?;

        let argument_list = if remaining.is_empty() {
            "void"
        } else {
            &demangle_argument_list(config, remaining, None)?
        };

        Ok(format!(
            "{template}::{method_name}({argument_list}){suffix}"
        ))
    } else if let Some(q_less) = remaining.strip_prefix('Q') {
        let (remaining, namespaces, _trailing_namespace) = demangle_namespaces(config, q_less)?;

        let argument_list = if remaining.is_empty() {
            "void"
        } else {
            &demangle_argument_list(config, remaining, Some(&namespaces))?
        };

        Ok(format!(
            "{namespaces}::{method_name}({argument_list}){suffix}"
        ))
    } else {
        let (remaining, class_name) = demangle_custom_name(remaining)?;

        let argument_list = if remaining.is_empty() {
            "void"
        } else {
            &demangle_argument_list(config, remaining, Some(class_name))?
        };

        Ok(format!(
            "{class_name}::{method_name}({argument_list}){suffix}"
        ))
    }
}

fn demangle_namespaced_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, namespaces, _trailing_namespace) = demangle_namespaces(config, s)?;

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, remaining, Some(&namespaces))?
    };

    let out = format!("{namespaces}::{func_name}({argument_list})");
    Ok(out)
}

fn demangle_type_info_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if let (remaining, DemangledArg::Plain(demangled_type)) =
        demangle_argument(config, s, None, &[])?
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
        demangle_argument(config, s, None, &[])?
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
            let (r, template, _typ) = demangle_template(config, r)?;

            stuff.push(template);
            r
        } else if let Some(r) = remaining.strip_prefix('Q') {
            let (r, namespaces, _trailing_namespace) = demangle_namespaces(config, r)?;

            stuff.push(namespaces);
            r
        } else {
            let (r, class_name) = demangle_custom_name(remaining)?;

            stuff.push(class_name.to_string());
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
        let (r, template, _typ) = demangle_template(config, r)?;

        (r, template)
    } else if let Some(r) = remaining.strip_prefix('Q') {
        let (r, namespaces, _trailing_namespace) = demangle_namespaces(config, r)?;

        (r, namespaces)
    } else {
        let (r, class_name) = demangle_custom_name(remaining)?;

        (r, class_name.to_string())
    };

    if !r.is_empty() {
        return Err(DemangleError::TrailingDataOnNamespacedGlobal(r));
    }

    Ok(format!("{space}::{name}"))
}

fn demangle_global_sym_keyed<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, which, is_constructor) = if let Some(r) = s.strip_prefix("I$") {
        (r, "constructors", true)
    } else if let Some(r) = s.strip_prefix("D$") {
        (r, "destructors", false)
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

    let actual_sym = demangled_sym.unwrap_or_else(|_| remaining.to_string());

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
        let (remaining, template, _typ) = demangle_template(config, remaining)?;

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
                demangle_namespaces(config, &args[1..])?;
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
        't' => {
            // templates
            let (remaining, template, _typ) = demangle_template(config, &args[1..])?;

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
                let (r, arg) = demangle_argument(config, args, namespace, &subargs)?;

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
                demangle_argument(config, args, namespace, &subargs)?
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
) -> Result<(&'s str, String, String), DemangleError<'s>> {
    let (remaining, namespace_count) = if let Some(r) = s.strip_prefix('_') {
        // More than a single digit of namespaces
        parse_number(r).and_then(|(r, l)| r.strip_prefix('_').map(|new_r| (new_r, l)))
    } else {
        parse_digit(s)
    }
    .ok_or(DemangleError::InvalidNamespaceCount(s))?;
    let namespace_count =
        NonZeroUsize::new(namespace_count).ok_or(DemangleError::InvalidNamespaceCount(s))?;

    demangle_namespaces_impl(config, remaining, namespace_count)
}

fn demangle_namespaces_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespace_count: NonZeroUsize,
) -> Result<(&'s str, String, String), DemangleError<'s>> {
    let mut namespaces = String::new();
    let mut remaining = s;
    let mut trailing_namespace = "".to_string();

    for _ in 0..namespace_count.get() {
        if !namespaces.is_empty() {
            namespaces.push_str("::");
        }

        let r = if let Some(temp) = remaining.strip_prefix('t') {
            let (r, template, _typ) = demangle_template(config, temp)?;
            trailing_namespace = template;
            r
        } else {
            let (r, ns) = demangle_custom_name(remaining)?;
            trailing_namespace = ns.to_string();
            r
        };
        remaining = r;
        namespaces.push_str(&trailing_namespace);
    }

    Ok((remaining, namespaces, trailing_namespace))
}

fn demangle_template<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let (remaining, class_name) = demangle_custom_name(s)?;
    let (mut remaining, digit) = parse_digit(remaining).unwrap();

    let namespace = None;
    let mut types = Vec::new();

    for _ in 0..digit {
        let r = if let Some(r) = remaining.strip_prefix('Z') {
            let (r, arg) = demangle_argument(config, r, namespace, &types)?;

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
                let (aux, DemangledArg::Plain(_arg)) = demangle_argument(config, r, None, &[])?
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

    let template = if types.last().is_some_and(|x| x.ends_with('>')) {
        format!("{}<{} >", class_name, types.join(", "))
    } else {
        format!("{}<{}>", class_name, types.join(", "))
    };
    Ok((remaining, template, class_name))
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
