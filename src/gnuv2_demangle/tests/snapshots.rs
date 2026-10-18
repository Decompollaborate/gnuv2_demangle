/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

// Run with this command, then see the diff in a git diff client.
// cargo insta test --accept

use std::borrow::Cow;

use gnuv2_demangle::{demangle, DemangleConfig, DemangleError};

fn demangle_lines<'s>(
    contents: &'s str,
    config: &DemangleConfig,
) -> Vec<(&'s str, Result<String, DemangleError<'s>>)> {
    contents
        .lines()
        .map(|line| (line, demangle(line, config).map(|x| x.to_string())))
        .collect()
}

/// Demangle each line or keep the raw line
///
/// Helps having an easy way to know if there are actually any changes on the
/// c++filt snapshots.
fn demangle_lines_plain<'s>(
    contents: &'s str,
    config: &DemangleConfig,
) -> Vec<(&'s str, Cow<'s, str>)> {
    contents
        .lines()
        .map(|line| {
            (
                line,
                demangle(line, config).map_or_else(|_| Cow::from(line), |x| Cow::from(x)),
            )
        })
        .collect()
}

#[test]
fn snapshot_mangled_list_hit_and_run_cfilt() {
    let contents = include_str!("mangled_lists/hit_and_run.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_hit_and_run_cfilt_plain() {
    let contents = include_str!("mangled_lists/hit_and_run.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines_plain(contents, &config));
}

#[test]
fn snapshot_mangled_list_hit_and_run_improved() {
    let contents = include_str!("mangled_lists/hit_and_run.txt");
    let config = DemangleConfig::new_g2dem();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_parappa2_cfilt() {
    let contents = include_str!("mangled_lists/parappa2.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_parappa2_cfilt_plain() {
    let contents = include_str!("mangled_lists/parappa2.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines_plain(contents, &config));
}

#[test]
fn snapshot_mangled_list_parappa2_improved() {
    let contents = include_str!("mangled_lists/parappa2.txt");
    let config = DemangleConfig::new_g2dem();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_ty_july_first_cfilt() {
    let contents = include_str!("mangled_lists/ty_july_first.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_ty_july_first_cfilt_plain() {
    let contents = include_str!("mangled_lists/ty_july_first.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines_plain(contents, &config));
}

#[test]
fn snapshot_mangled_list_ty_july_first_improved() {
    let contents = include_str!("mangled_lists/ty_july_first.txt");
    let config = DemangleConfig::new_g2dem();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_ff2_cfilt() {
    let contents = include_str!("mangled_lists/ff2.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_ff2_cfilt_plain() {
    let contents = include_str!("mangled_lists/ff2.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines_plain(contents, &config));
}

#[test]
fn snapshot_mangled_list_ff2_improved() {
    let contents = include_str!("mangled_lists/ff2.txt");
    let config = DemangleConfig::new_g2dem();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_most_wanted_cfilt() {
    let contents = include_str!("mangled_lists/most_wanted.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}

#[test]
fn snapshot_mangled_list_most_wanted_cfilt_plain() {
    let contents = include_str!("mangled_lists/most_wanted.txt");
    let config = DemangleConfig::new_cfilt();

    insta::assert_debug_snapshot!(demangle_lines_plain(contents, &config));
}

#[test]
fn snapshot_mangled_list_most_wanted_improved() {
    let contents = include_str!("mangled_lists/most_wanted.txt");
    let config = DemangleConfig::new_g2dem();

    insta::assert_debug_snapshot!(demangle_lines(contents, &config));
}
