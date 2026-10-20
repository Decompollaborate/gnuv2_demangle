/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, sync::Arc};

use crate::{
    dem_leaf::dem_arg_list::demangle_argument_list,
    tree::{ArgVec, NamespacedType, NamespacedTypeClassMethod, StaticSuffix},
    DemangleConfig, DemangleError,
};

// Maybe merge with demangle_method
pub(crate) fn demangle_namespaced_type_class_method<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespaced_type: NamespacedType<'s>,
    allow_array_fixup: bool,
) -> Result<NamespacedTypeClassMethod<'s>, DemangleError<'s>> {
    let remaining = s;

    let method_name = namespaced_type.type_name;
    let namespaced_type = Arc::new(namespaced_type);

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            Some(Arc::clone(&namespaced_type)),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    let out =
        NamespacedTypeClassMethod::new(method_name, namespaced_type, argument_list, static_suffix);
    Ok(out)
}
