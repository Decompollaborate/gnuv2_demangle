/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    dem_leaf::dem_namespaced_type::demangle_namespaced_type, remainer::Remaining,
    tree::NamespacedGlobal, DemangleConfig, DemangleError,
};

pub(crate) fn demangle_namespaced_global<'s>(
    config: &DemangleConfig,
    s: &'s str,
    name: &'s str,
) -> Result<NamespacedGlobal<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let Some(remaining) = s.strip_prefix('_') else {
        return Err(DemangleError::InvalidNamespacedGlobal(s, name));
    };

    let Remaining { r, d: namespace } = demangle_namespaced_type(
        config,
        remaining,
        allow_array_fixup,
        DemangleError::InvalidNamespaceOnNamespacedGlobal,
    )?
    .ok_or(DemangleError::InvalidTypeNameForNamespacedGlobal(remaining))?;

    if !r.is_empty() {
        return Err(DemangleError::TrailingDataOnNamespacedGlobal(r));
    }

    let nsg = NamespacedGlobal::new(namespace, name);
    Ok(nsg)
}
