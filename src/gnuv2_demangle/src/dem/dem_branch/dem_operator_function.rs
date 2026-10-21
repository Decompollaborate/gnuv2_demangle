/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    dem::dem_leaf::demangle_argument_list,
    tree::{
        branch::OperatorFunction,
        leaf::{Operator, TemplatedValueVec},
    },
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_operator_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
    operator: Operator,
    allow_array_fixup: bool,
) -> Result<OperatorFunction<'s>, DemangleError<'s>> {
    let (argument_list, static_suffix) = demangle_argument_list(
        config,
        s,
        None,
        &TemplatedValueVec::new(),
        allow_array_fixup,
    )?;

    let of = OperatorFunction::new(operator, argument_list, static_suffix);
    Ok(of)
}
