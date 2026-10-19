/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, string::String};

use crate::{
    dem_leaf::dem_arg_list::demangle_argument_list,
    tree::{ArgVec, NamespacedType, StaticSuffix},
    DemangleConfig, DemangleError,
};

// Maybe merge with demangle_method
pub(crate) fn demangle_namespaced_type_class_method<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespaced_type: NamespacedType<'s>,
    allow_array_fixup: bool,
) -> Result<String, DemangleError<'s>> {
    let remaining = s;

    let NamespacedType {
        namespace: class_name,
        type_name: method_name,
    } = namespaced_type;

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            Some(&class_name),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    let out = format!("{class_name}::{method_name}{static_suffix}({argument_list})");

    Ok(out)
}
