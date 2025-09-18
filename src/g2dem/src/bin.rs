/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

#![doc = include_str!("../README.md")]

use std::io::{self, BufRead};

use argp::FromArgs;
use gnuv2_demangle::{demangle, DemangleConfig};

/// GNUv2 C++ symbol demangler
#[derive(FromArgs)]
struct Args {
    /// List of symbols to demangle.
    ///
    /// If no symbols are passed then demangles symbols from stdin. Remember to escape `$`.
    #[argp(positional)]
    syms: Vec<String>,
}

fn main() {
    let args: Args = argp::parse_args_or_exit(argp::DEFAULT);
    let config = DemangleConfig::new();

    if args.syms.is_empty() {
        for line in io::stdin().lock().lines() {
            let line = line.expect("Error reading from stdin");

            if let Ok(demangled) = demangle(&line, &config) {
                println!("{demangled}");
            } else {
                println!("{line}");
            }
        }
    } else {
        for mangled in args.syms {
            if let Ok(demangled) = demangle(&mangled, &config) {
                println!("{demangled}");
            } else {
                println!("{mangled}");
            }
        }
    }
}
