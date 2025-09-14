/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
#![no_std]

extern crate alloc;

use alloc::string::String;
*/

use core::num::NonZeroUsize;

mod demangle_error;

pub use demangle_error::DemangleError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct DemangleConfig {}

impl DemangleConfig {
    pub fn new() -> Self {
        // The default config should mimic c++filt's behavior.
        Self {}
    }
}

impl Default for DemangleConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn demangle<'s>(sym: &'s str, config: &DemangleConfig) -> Result<String, DemangleError<'s>> {
    if !sym.is_ascii() {
        return Err(DemangleError::NonAscii);
    }

    if let Some(s) = sym.strip_prefix("_$_") {
        let (remaining, class_name, suffix) = demangle_custom_name(s)?;

        if remaining.is_empty() {
            Ok(format!("{class_name}::~{class_name}(void){suffix}"))
        } else {
            Err(DemangleError::TrailingData)
        }
    } else if let Some(s) = sym.strip_prefix("__") {
        demangle_special(config, s)
    } else if let Some((func_name, args)) = str_split_2(sym, "__F") {
        demangle_free_function(config, func_name, args)
    } else if let Some((method_name, class_and_args)) =
        str_split_2_second_starts_with(sym, "__", |c| matches!(c, '1'..='9' | 'C'))
    {
        demangle_method(config, method_name, class_and_args)
    } else if let Some(q_index) = sym.find("__Q") {
        demangle_namespaced_function(config, &sym[..q_index], &sym[q_index + 3..])
    } else {
        Err(DemangleError::NotMangled)
    }
}

fn str_split_2<'a>(s: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    let mut iter = s.splitn(2, pat);

    if let (Some(l), Some(r)) = (iter.next(), iter.next()) {
        Some((l, r))
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
        if r.starts_with(second_start) {
            Some((l, r))
        } else {
            None
        }
    } else {
        None
    }
}

fn demangle_special<'s>(config: &DemangleConfig, s: &'s str) -> Result<String, DemangleError<'s>> {
    let c = s
        .chars()
        .next()
        .ok_or(DemangleError::RanOutWhileDemanglingSpecial)?;

    let (s, class_name, method_name, suffix) = if matches!(c, '1'..='9') {
        // class constructor
        let (s, class_name, suffix) = demangle_custom_name(s)?;

        (s, Some(class_name), class_name, suffix)
    } else {
        let end_index = s.find("__").ok_or(DemangleError::InvalidSpecialMethod(s))?;
        let op = &s[..end_index];

        let method_name = match op {
            "nw" => "operator new",
            "dl" => "operator delete",
            "vn" => "operator new []",
            "eq" => "operator==",
            "ne" => "operator!=",
            "as" => "operator=",
            _ => return Err(DemangleError::UnrecognizedSpecialMethod(op)),
        };

        let s = &s[end_index + 2..];

        if let Some(s) = s.strip_prefix('F') {
            (s, None, method_name, "")
        } else {
            let (s, class_name, suffix) = demangle_custom_name(s)?;

            (s, Some(class_name), method_name, suffix)
        }
    };

    let argument_list = if s.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, s)?
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
    let argument_list = demangle_argument_list(config, args)?;

    Ok(format!("{func_name}({argument_list})"))
}

fn demangle_argument_list<'s>(
    _config: &DemangleConfig,
    mut args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let mut demangled = String::new();

    let mut first = true;
    while !args.is_empty() {
        let (a, b) = demangle_argument(args)?;
        args = a;
        if !first {
            demangled.push_str(", ");
        }
        first = false;
        demangled.push_str(&b);
    }

    Ok(demangled)
}

fn demangle_method<'s>(
    config: &DemangleConfig,
    method_name: &'s str,
    class_and_args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (s, class_name, suffix) = demangle_custom_name(class_and_args)?;

    let argument_list = if s.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, s)?
    };

    Ok(format!(
        "{class_name}::{method_name}({argument_list}){suffix}"
    ))
}

fn demangle_namespaced_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, namespace_count) = if let Some(r) = s.strip_prefix('_') {
        // More than a single digit of namespaces
        parse_number(r).and_then(|(r, l)| r.strip_prefix('_').map(|new_r| (new_r, l)))
    } else {
        parse_digit(s)
    }
    .ok_or(DemangleError::InvalidNamespaceCount(s))?;
    let namespace_count =
        NonZeroUsize::new(namespace_count).ok_or(DemangleError::InvalidNamespaceCount(s))?;

    let (remaining, namespaces) = demangle_namespaces(remaining, namespace_count)?;

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(config, remaining)?
    };

    let out = format!("{namespaces}{func_name}({argument_list})");
    Ok(out)
}

fn demangle_argument<'s>(full_args: &'s str) -> Result<(&'s str, String), DemangleError<'s>> {
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
            let (remaining, class_name, _suffix) = demangle_custom_name(args)?;
            args = remaining;
            is_class_like = true;
            out.push_str(class_name);
        }
        _ => {
            return Err(DemangleError::UnknownType(c));
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

    Ok((args, out))
}

fn demangle_namespaces<'s>(
    s: &'s str,
    namespace_count: NonZeroUsize,
) -> Result<(&'s str, String), DemangleError<'s>> {
    let mut namespaces = String::new();
    let mut remaining = s;

    for _ in 0..namespace_count.get() {
        let (r, ns, _suffix) = demangle_custom_name(remaining)?;
        remaining = r;
        namespaces.push_str(ns);
        namespaces.push_str("::");
    }

    Ok((remaining, namespaces))
}

fn demangle_custom_name<'s>(
    s: &'s str,
) -> Result<(&'s str, &'s str, &'static str), DemangleError<'s>> {
    let (remaining, suffix) = if let Some(remaining) = s.strip_prefix('C') {
        (remaining, " const")
    } else {
        (s, "")
    };

    let (remaining, length) =
        parse_number(remaining).ok_or(DemangleError::InvalidCustomNameCount(s))?;

    if remaining.len() < length {
        Err(DemangleError::RanOutOfCharactersForCustomName(s))
    } else {
        Ok((&remaining[length..], &remaining[..length], suffix))
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
