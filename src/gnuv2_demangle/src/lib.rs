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

    if let Some((func_name, mut args)) = str_split_2(sym, "__F") {
        // Arbitrarily large value
        let mut demangled = String::with_capacity(sym.len() * 4);
        demangled.push_str(func_name);
        demangled.push('(');
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
        demangled.push(')');

        Some(demangled)
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

fn demangle_argument(mut args: &str) -> Option<(&str, String)> {
    let mut out = String::new();
    let mut post = String::new();

    // Qualifiers
    while !args.is_empty() {
        let c = args.chars().next()?;

        match c {
            'P' => post.insert(0, '*'),
            'C' => post.insert_str(0, "const "),
            'S' => out.push_str("signed "),
            'U' => out.push_str("unsigned "),
            _ => break,
        }

        args = &args[1..];
    }

    // Plain types
    match args.chars().next()? {
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
        _ => {
            return None;
        }
    }

    args = &args[1..];

    let out = if !post.is_empty() {
        out + " " + post.trim_matches(' ')
    } else {
        out
    };
    // out.trim_matches(' ').to_string()

    Some((args, out))
}
