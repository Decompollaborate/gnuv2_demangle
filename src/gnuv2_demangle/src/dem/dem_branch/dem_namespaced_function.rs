/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::sync::Arc;

use crate::{
    dem::dem_leaf::{demangle_argument_list, demangle_namespaces},
    remainer::Remaining,
    tree::{branch::NamespacedFunction, leaf::TemplatedValueVec},
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
    } = demangle_namespaces(config, s, &TemplatedValueVec::new(), allow_array_fixup)?;
    let ns = Arc::new(ns);

    let (argument_list, static_suffix) = demangle_argument_list(
        config,
        remaining,
        Some(Arc::clone(&ns)),
        &TemplatedValueVec::new(),
        allow_array_fixup,
    )?;

    let nsf = NamespacedFunction::new(func_name, argument_list, ns, static_suffix);
    Ok(nsf)
}
