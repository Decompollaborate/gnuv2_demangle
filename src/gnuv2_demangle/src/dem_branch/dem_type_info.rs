/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    dem_leaf::dem_arg::demangle_argument,
    tree::{ArgVec, DemangledArg, TypeInfoFunction, TypeInfoNode},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_type_info_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<TypeInfoFunction<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (remaining, DemangledArg::Plain(demangled_type, array_qualifiers)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &ArgVec::new(config, None),
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidTypeOnTypeInfoFunction(s));
    };

    if remaining.is_empty() {
        Ok(TypeInfoFunction::new(demangled_type, array_qualifiers))
    } else {
        Err(DemangleError::TrailingDataOnTypeInfoFunction(remaining))
    }
}

pub(crate) fn demangle_type_info_node<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<TypeInfoNode<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (remaining, DemangledArg::Plain(demangled_type, array_qualifiers)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &ArgVec::new(config, None),
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidTypeOnTypeInfoNode(s));
    };

    if remaining.is_empty() {
        Ok(TypeInfoNode::new(demangled_type, array_qualifiers))
    } else {
        Err(DemangleError::TrailingDataOnTypeInfoNode(remaining))
    }
}
