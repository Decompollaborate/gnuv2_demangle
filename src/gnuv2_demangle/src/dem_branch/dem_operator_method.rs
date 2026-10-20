/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::borrow::Cow;

use crate::{
    dem_leaf::{
        dem::demangle_method_qualifier, dem_arg_list::demangle_argument_list,
        dem_namespaced_type::demangle_namespaced_type,
    },
    remainer::Remaining,
    tree::{ArgVec, Operator, OperatorMethod, StaticSuffix},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_operator_method<'s>(
    config: &DemangleConfig,
    s: &'s str,
    operator: Operator,
    allow_array_fixup: bool,
) -> Result<OperatorMethod<'s>, DemangleError<'s>> {
    let Remaining {
        r: remaining,
        d: method_qualifier,
    } = demangle_method_qualifier(s);

    let Remaining {
        r: remaining,
        d: namespace,
    } = demangle_namespaced_type(
        config,
        remaining,
        allow_array_fixup,
        DemangleError::InvalidClassNameOnOperator,
    )?
    .ok_or(DemangleError::InvalidTypeNameForOperator(remaining))?;

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

    let om = OperatorMethod::new(
        operator,
        method_qualifier,
        namespace,
        argument_list,
        static_suffix,
    );
    Ok(om)
}
