/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::borrow::Cow;

use crate::{
    dem_leaf::{
        dem::demangle_method_qualifier, dem_arg_list::demangle_argument_list,
        dem_namespaced_type::demangle_namespaced_type,
    },
    remainer::Remaining,
    tree::{ArgVec, NamespacedType, Operator, OperatorMethod, StaticSuffix},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_operator_method<'s>(
    config: &DemangleConfig,
    s: &'s str,
    operator: Operator,
    allow_array_fixup: bool,
) -> Result<OperatorMethod<'s>, DemangleError<'s>> {
    let remaining = s;

    let (remaining, class_name, method_name, suffix) = {
        let Remaining {
            r: remaining,
            d: suffix,
        } = demangle_method_qualifier(remaining);

        let Remaining {
            r: remaining,
            d:
                NamespacedType {
                    namespace,
                    type_name: _,
                },
        } = demangle_namespaced_type(
            config,
            remaining,
            allow_array_fixup,
            DemangleError::InvalidClassNameOnOperator,
        )?
        .ok_or(DemangleError::InvalidTypeNameForOperator(remaining))?;

        (remaining, Some(namespace), operator, suffix)
    };

    let (argument_list, static_suffix) = if remaining.is_empty() {
        (Cow::from("void"), StaticSuffix::none())
    } else {
        demangle_argument_list(
            config,
            remaining,
            class_name.as_ref(),
            &ArgVec::new(config, None),
            allow_array_fixup,
        )?
    };

    let out = if let Some(class_name) = class_name {
        format!("{class_name}::{method_name}{static_suffix}({argument_list}){suffix}")
    } else {
        format!("{method_name}{static_suffix}({argument_list}){suffix}")
    };

    let om = OperatorMethod::new(out);
    Ok(om)
}
