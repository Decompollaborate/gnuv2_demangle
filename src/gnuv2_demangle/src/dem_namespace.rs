/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroUsize;

use alloc::{borrow::Cow, string::String};

use crate::{DemangleConfig, DemangleError};

use crate::{
    dem::demangle_custom_name,
    dem_arg_list::ArgVec,
    dem_template::demangle_template,
    remainer::{Remaining, StrParsing},
};

// 'Q' must be stripped already
pub(crate) fn demangle_namespaces<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &ArgVec,
    allow_array_fixup: bool,
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

    demangle_namespaces_impl(config, r, namespace_count, template_args, allow_array_fixup)
}

fn demangle_namespaces_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespace_count: NonZeroUsize,
    template_args: &ArgVec,
    allow_array_fixup: bool,
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let mut namespaces = String::new();
    let mut remaining = s;
    let mut trailing_type = "";

    for _i in 0..namespace_count.get() {
        if !namespaces.is_empty() {
            namespaces.push_str("::");
        }

        // Sometimes there's a trailing underscore after a number.
        // Not sure if this is the correct way to handle this, but at least it
        // doesn't seem to break anything else.
        // i.e. CreateRoadBlock__12AICopManagerP8IPursuitiP8IVehiclePQ43UTL11Collectionst11ListableSet4Z8IVehiclei10Z12eVehicleListUi10_4List
        remaining = remaining.trim_start_matches('_');

        let (r, n) = if let Some(temp) = remaining.strip_prefix('t') {
            let (r, template, typ) =
                demangle_template(config, temp, template_args, allow_array_fixup)?;
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
