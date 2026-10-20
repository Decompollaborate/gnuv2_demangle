/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    dem::dem_leaf::{demangle_namespaced_type, demangle_static_suffix},
    remainer::Remaining,
    tree::{branch::Destructor, leaf::NamespacedType},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_destructor<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<Destructor<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let Remaining {
        r,
        d: NamespacedType {
            namespace,
            type_name,
        },
    } = demangle_namespaced_type(
        config,
        s,
        allow_array_fixup,
        DemangleError::InvalidClassNameOnDestructor,
    )?
    .ok_or(DemangleError::InvalidTypeNameForDestructor(s))?;

    let Remaining {
        r,
        d: static_suffix,
    } = demangle_static_suffix(config, r)?;

    if r.is_empty() {
        Ok(Destructor::new(namespace, type_name, static_suffix))
    } else {
        Err(DemangleError::TrailingDataOnDestructor(r))
    }
}
