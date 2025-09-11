/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
#![no_std]

extern crate alloc;

use alloc::string::String;
*/

mod demangle_error;

pub use demangle_error::DemangleError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct DemangleOptions {
    pub try_recover_on_failure: bool,
}

impl DemangleOptions {
    pub fn new() -> Self {
        Self {
            try_recover_on_failure: false,
        }
    }
}

impl Default for DemangleOptions {
    fn default() -> Self {
        Self::new()
    }
}

pub fn demangle<'s>(sym: &'s str, options: &DemangleOptions) -> Result<String, DemangleError<'s>> {
    if !sym.is_ascii() {
        return Err(DemangleError::NonAscii);
    }

    if let Some(s) = sym.strip_prefix("_$_") {
        let (remaining, class_name) = demangle_class_name(s)?;

        if remaining.is_empty() {
            Ok(format!("{class_name}::~{class_name}(void)"))
        } else {
            Err(DemangleError::TrailingData)
        }
    } else if let Some(s) = sym.strip_prefix("__") {
        demangle_constructor(options, s)
    } else if let Some((func_name, args)) = str_split_2(sym, "__F") {
        demangle_free_function(options, func_name, args)
    } else if let Some((method_name, class_and_args)) =
        str_split_2_second_starts_with(sym, "__", |c| matches!(c, '1'..='9' | 'C'))
    {
        demangle_method(options, method_name, class_and_args)
    } else {
        Err(DemangleError::Invalid)
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

fn demangle_constructor<'s>(
    options: &DemangleOptions,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (s, class_name) = demangle_class_name(s)?;

    let argument_list = if s.is_empty() {
        "void"
    } else {
        &demangle_argument_list(options, s)?
    };

    Ok(format!("{class_name}::{class_name}({argument_list})"))
}

fn demangle_free_function<'s>(
    options: &DemangleOptions,
    func_name: &'s str,
    args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let argument_list = demangle_argument_list(options, args)?;

    Ok(format!("{func_name}({argument_list})"))
}

fn demangle_argument_list<'s>(
    options: &DemangleOptions,
    mut args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let mut demangled = String::new();

    let mut first = true;
    while !args.is_empty() {
        let (a, b) = match demangle_argument(args) {
            Ok((a, b)) => (a, b),
            Err(e) => {
                return if options.try_recover_on_failure {
                    Ok(demangled)
                } else {
                    Err(e)
                };
            }
        };
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
    options: &DemangleOptions,
    method_name: &'s str,
    mut class_and_args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let suffix = if class_and_args.starts_with('C') {
        class_and_args = &class_and_args[1..];
        " const"
    } else {
        ""
    };

    let (s, class_name) = demangle_class_name(class_and_args)?;

    let argument_list = if s.is_empty() {
        "void"
    } else {
        &demangle_argument_list(options, s)?
    };

    Ok(format!(
        "{class_name}::{method_name}({argument_list}){suffix}"
    ))
}

fn demangle_argument<'s>(mut args: &'s str) -> Result<(&'s str, String), DemangleError<'s>> {
    let mut out = String::new();
    let mut post = String::new();

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

    let mut c = args
        .chars()
        .next()
        .ok_or(DemangleError::RanOutOfArguments)?;
    if c == 'G' {
        // TODO: figure out what does 'G' mean
        args = &args[1..];
        c = args
            .chars()
            .next()
            .ok_or(DemangleError::RanOutOfArguments)?;
    }

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
            let (remaining, class_name) = demangle_class_name(args)?;
            args = remaining;
            out.push_str(class_name);
        }
        _ => {
            return Err(DemangleError::UnknownType(c));
        }
    }

    if !args.is_empty() {
        args = &args[1..];
    }

    let out = if !post.is_empty() {
        out + " " + post.trim_matches(' ')
    } else {
        out
    };

    Ok((args, out))
}

fn demangle_class_name<'s>(s: &'s str) -> Result<(&'s str, &'s str), DemangleError<'s>> {
    let (remaining, length) = parse_number(s).ok_or(DemangleError::InvalidClassName(s))?;

    if remaining.len() < length {
        Err(DemangleError::InvalidClassName(s))
    } else {
        Ok((&remaining[length..], &remaining[..length]))
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
