/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use gnuv2_demangle::{demangle, DemangleConfig, DemangleError};

fn demangle_lines<'s>(
    contents: &'s str,
    config: &DemangleConfig,
) -> Vec<Result<String, DemangleError<'s>>> {
    contents
        .lines()
        .map(|line| demangle(line, config))
        .collect()
}

#[test]
fn snapshot_mangled_list_hit_and_run_cfilt() {
    let contents = include_str!("mangled_lists/hit_and_run.txt");
    let config = DemangleConfig::new_mimic_cfilt();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_hit_and_run_improved() {
    let contents = include_str!("mangled_lists/hit_and_run.txt");
    let config = DemangleConfig::new_no_cfilt_mimics();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_parappa2_cfilt() {
    let contents = include_str!("mangled_lists/parappa2.txt");
    let config = DemangleConfig::new_mimic_cfilt();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_parappa2_improved() {
    let contents = include_str!("mangled_lists/parappa2.txt");
    let config = DemangleConfig::new_no_cfilt_mimics();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}
