/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;

use crate::{
    dem_leaf::dem_namespaced_type::demangle_namespaced_type, remainer::Remaining,
    tree::VirtualTable, DemangleConfig, DemangleError,
};

pub(crate) fn demangle_virtual_table<'s>(
    config: &DemangleConfig,
    s: &'s str,
    cplus_marker: char,
) -> Result<VirtualTable<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;
    let mut remaining = s;
    let mut namespaces = Vec::new();

    while !remaining.is_empty() {
        remaining = remaining
            .strip_prefix(cplus_marker)
            .ok_or(DemangleError::VTableMissingCPlusMarker(remaining))?;

        let Remaining { r, d: ns } = demangle_namespaced_type(
            config,
            remaining,
            allow_array_fixup,
            DemangleError::InvalidClassNameOnVirtualTable,
        )?
        .ok_or(DemangleError::InvalidTypeNameForVirtualTable(remaining))?;

        namespaces.push(ns);
        remaining = r;
    }

    if namespaces.is_empty() {
        return Err(DemangleError::MissingTypesForVirtualTable(s));
    }

    let vt = VirtualTable::new(namespaces);
    Ok(vt)
}
