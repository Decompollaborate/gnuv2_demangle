/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
#![no_std]

extern crate alloc;

use alloc::string::String;
*/

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

pub fn demangle(sym: &str, options: &DemangleOptions) -> Option<String> {
    if !sym.is_ascii() {
        return None;
    }

    if let Some(s) = sym.strip_prefix("_$_") {
        let (remaining, class_name) = demangle_class_name(s)?;

        if remaining.is_empty() {
            Some(format!("{class_name}::~{class_name}(void)"))
        } else {
            None
        }
    } else if let Some(s) = sym.strip_prefix("__") {
        demangle_constructor(options, s)
    } else if let Some((func_name, args)) = str_split_2(sym, "__F") {
        demangle_free_function(options, func_name, args)
    } else {
        None
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

fn demangle_constructor(options: &DemangleOptions, s: &str) -> Option<String> {
    let (s, class_name) = demangle_class_name(s)?;

    let argument_list = if s.is_empty() {
        "void"
    } else {
        &demangle_argument_list(options, s)?
    };

    Some(format!("{class_name}::{class_name}({argument_list})"))
}

fn demangle_free_function(
    options: &DemangleOptions,
    func_name: &str,
    args: &str,
) -> Option<String> {
    let argument_list = demangle_argument_list(options, args)?;

    Some(format!("{func_name}({argument_list})"))
}

fn demangle_argument_list(options: &DemangleOptions, mut args: &str) -> Option<String> {
    let mut demangled = String::new();

    let mut first = true;
    while !args.is_empty() {
        let Some((a, b)) = demangle_argument(args) else {
            return if options.try_recover_on_failure {
                Some(demangled)
            } else {
                None
            };
        };
        args = a;
        if !first {
            demangled.push_str(", ");
        }
        first = false;
        demangled.push_str(&b);
    }

    Some(demangled)
}

fn demangle_argument(mut args: &str) -> Option<(&str, String)> {
    let mut out = String::new();
    let mut post = String::new();

    // Qualifiers
    while !args.is_empty() {
        let c = args.chars().next()?;

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

    let mut c = args.chars().next()?;
    if c == 'G' {
        // TODO: figure out what does 'G' mean
        args = &args[1..];
        c = args.chars().next()?
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
        '1'..'9' => {
            let (remaining, class_name) = demangle_class_name(args)?;
            args = remaining;
            out.push_str(class_name);
        }
        _ => {
            return None;
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

    Some((args, out))
}

fn demangle_class_name(s: &str) -> Option<(&str, &str)> {
    let (s, length) = parse_number(s)?;

    if s.len() < length {
        None
    } else {
        Some((&s[length..], &s[..length]))
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
