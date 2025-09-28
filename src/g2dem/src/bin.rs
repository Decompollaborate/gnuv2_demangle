/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

#![doc = include_str!("../README.md")]

use std::io::{self, BufRead};

use argp::FromArgs;
use gnuv2_demangle::{demangle, DemangleConfig};

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// GNUv2 C++ symbol demangler
#[derive(FromArgs)]
struct Args {
    /// List of symbols to demangle.
    ///
    /// If no symbols are passed then demangles symbols from stdin. Remember to escape `$`.
    #[argp(positional)]
    syms: Vec<String>,

    /// Print current version information and exit.
    #[argp(switch, short = 'V')]
    version: bool,
}

fn show_version() {
    let (dirty, hash_short) = if built_info::GIT_DIRTY == Some(true) {
        let hash_short = built_info::GIT_COMMIT_HASH_SHORT.unwrap_or("");
        (" (dirty) ", hash_short)
    } else {
        ("", "")
    };
    println!(
        "{} {}{}{}",
        built_info::PKG_NAME,
        built_info::PKG_VERSION,
        dirty,
        hash_short
    );
    println!();

    if let (Some(git_version), Some(git_hash_short)) =
        (built_info::GIT_VERSION, built_info::GIT_COMMIT_HASH_SHORT)
    {
        // If the current commit is tagged then `git_version` will end with
        // that tag name, otherwise it ends with the short hash.
        if git_version.ends_with(git_hash_short) {
            println!("Untagged git info: {}", git_version);
        } else {
            // empty
        }
    } else {
        println!("No git information?");
    }

    println!("Built time (UTC): {}", built_info::BUILT_TIME_UTC);
    println!("Build profile: {}", built_info::PROFILE);
    println!("Repository: {}", built_info::PKG_REPOSITORY);

    if let Some(ci) = built_info::CI_PLATFORM {
        println!("Built on CI platform: {}", ci);
    } else {
        println!("Built locally.");
    }
}

fn main() {
    let args: Args = argp::parse_args_or_exit(argp::DEFAULT);

    if args.version {
        show_version();
        return;
    }

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
