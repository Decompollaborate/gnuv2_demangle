/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, sync::Arc};

use crate::{
    dem::dem_leaf::{demangle_argument_list, demangle_namespaces},
    remainer::Remaining,
    tree::{
        branch::NamespacedFunction,
        leaf::{ArgVec, StaticSuffix},
    },
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_namespaced_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<NamespacedFunction<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    // TODO: consider changing to demangle_namespaced_type
    let Remaining {
        r: remaining,
        d: ns,
    } = demangle_namespaces(config, s, &ArgVec::new(config, None), allow_array_fixup)?;
    let ns = Arc::new(ns);

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            Some(Arc::clone(&ns)),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    let nsf = NamespacedFunction::new(func_name, argument_list, ns, static_suffix);
    Ok(nsf)
}
