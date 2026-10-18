/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::borrow::Cow;

use crate::{
    dem_leaf::{
        dem::demangle_method_qualifier, dem_arg_list::demangle_argument_list,
        dem_namespaced_type::demangle_namespaced_type,
    },
    remainer::Remaining,
    tree::{ArgVec, Method, StaticSuffix},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_method<'s>(
    config: &DemangleConfig,
    method_name: &'s str,
    class_and_args: &'s str,
) -> Result<Method<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;
    let Remaining {
        r: remaining,
        d: method_qualifier,
    } = demangle_method_qualifier(class_and_args);

    let Remaining {
        r: remaining,
        d: namespace,
    } = demangle_namespaced_type(
        config,
        remaining,
        allow_array_fixup,
        DemangleError::InvalidClassNameOnMethod,
    )?
    .ok_or(DemangleError::InvalidTypeNameForMethod(remaining))?;

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            Some(&namespace.namespace),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    let m = Method::new(
        method_name,
        argument_list,
        method_qualifier,
        namespace,
        static_suffix,
    );
    Ok(m)
}
