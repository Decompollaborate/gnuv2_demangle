/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroUsize;

use alloc::{borrow::Cow, string::String};

use crate::{DemangleConfig, DemangleError};

use crate::{
    dem::demangle_custom_name,
    dem_arg::DemangledArgVec,
    dem_template::demangle_template,
    remainer::{Remaining, StrParsing},
};

// 'Q' must be stripped already
pub(crate) fn demangle_namespaces<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &DemangledArgVec,
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let Remaining {
        r,
        d: namespace_count,
    } = if let Some(r) = s.strip_prefix('_') {
        // More than a single digit of namespaces
        r.p_number().and_then(|Remaining { r, d }| {
            r.strip_prefix('_').map(|new_r| Remaining::new(new_r, d))
        })
    } else {
        s.p_digit()
    }
    .ok_or(DemangleError::InvalidNamespaceCount(s))?;

    let namespace_count =
        NonZeroUsize::new(namespace_count).ok_or(DemangleError::InvalidNamespaceCount(s))?;

    demangle_namespaces_impl(config, r, namespace_count, template_args)
}

fn demangle_namespaces_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespace_count: NonZeroUsize,
    template_args: &DemangledArgVec,
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let mut namespaces = String::new();
    let mut remaining = s;
    let mut trailing_type = "";

    for _ in 0..namespace_count.get() {
        if !namespaces.is_empty() {
            namespaces.push_str("::");
        }

        let (r, n) = if let Some(temp) = remaining.strip_prefix('t') {
            let (r, template, typ) = demangle_template(config, temp, template_args)?;
            trailing_type = typ;
            (r, Cow::from(template))
        } else {
            let Remaining { r, d: ns } =
                demangle_custom_name(remaining, DemangleError::InvalidCustomNameOnNamespace)?;
            trailing_type = ns;
            (r, Cow::from(ns))
        };
        remaining = r;
        namespaces.push_str(&n);
    }

    Ok((remaining, namespaces, trailing_type))
}
