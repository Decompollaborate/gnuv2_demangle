/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT */

use gnuv2_demangle::{DemangleOptions, demangle};

#[test]
fn test_demangling_funcs() {
    static CASES: [(&str, Option<&str>); 7] = [
        ("whatever_default__Fcsilx", Some("whatever_default(char, short, int, long, long long)")),
        ("whatever_signed__FScsilx", Some("whatever_signed(signed char, short, int, long, long long)")),
        ("whatever_unsigned__FUcUsUiUlx", Some("whatever_unsigned(unsigned char, unsigned short, unsigned int, unsigned long, long long)")),
        ("whatever_other__Ffdrb", Some("whatever_other(float, double, long double, bool)")),
        ("whatever_why__Fw", Some("whatever_why(wchar_t)")),
        ("whatever_pointer__FPcPsPiPlPx", Some("whatever_pointer(char *, short *, int *, long *, long long *)")),
        ("whatever_const_pointer__FPCcPCsPCiPClPCx", Some("whatever_const_pointer(char const *, short const *, int const *, long const *, long long const *)")),
    ];
    let mut options = DemangleOptions::new();
    options.try_recover_on_failure = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &options).as_deref(), demangled);
    }
}

#[test]
fn test_demangling_funcs_const_pointer_const() {
    static CASES: [(&str, Option<&str>); 5] = [
        (
            "whatever_const_pointer__FPc",
            Some("whatever_const_pointer(char *)"),
        ),
        (
            "whatever_const_pointer__FPCc",
            Some("whatever_const_pointer(char const *)"),
        ),
        (
            "whatever_const_pointer__FCPCc",
            Some("whatever_const_pointer(char const *const)"),
        ),
        (
            "whatever_const_pointer__FCPc",
            Some("whatever_const_pointer(char *const)"),
        ),
        (
            "silly_function__FPCPCPCPCPCc",
            Some("silly_function(char const *const *const *const *const *)"),
        ),
    ];
    let mut options = DemangleOptions::new();
    options.try_recover_on_failure = true;

    for (mangled, demangled) in CASES {
        assert_eq!(demangle(mangled, &options).as_deref(), demangled);
    }
}

