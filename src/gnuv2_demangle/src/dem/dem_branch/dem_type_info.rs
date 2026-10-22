/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    dem::dem_leaf::demangle_argument,
    tree::{
        branch::{TypeInfoFunction, TypeInfoNode},
        leaf::{ArgVec, DemangledArg, TemplatedValueVec},
    },
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_type_info_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<TypeInfoFunction<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (remaining, DemangledArg::Plain(demangled_type)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &TemplatedValueVec::new(),
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidTypeOnTypeInfoFunction(s));
    };

    if remaining.is_empty() {
        Ok(TypeInfoFunction::new(demangled_type))
    } else {
        Err(DemangleError::TrailingDataOnTypeInfoFunction(remaining))
    }
}

pub(crate) fn demangle_type_info_node<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<TypeInfoNode<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (remaining, DemangledArg::Plain(demangled_type)) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        &TemplatedValueVec::new(),
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidTypeOnTypeInfoNode(s));
    };

    if remaining.is_empty() {
        Ok(TypeInfoNode::new(demangled_type))
    } else {
        Err(DemangleError::TrailingDataOnTypeInfoNode(remaining))
    }
}
