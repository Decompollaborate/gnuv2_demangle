/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    dem::dem_leaf::demangle_argument_list,
    tree::{branch::FreeFunction, leaf::TemplatedValueVec},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_free_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    args: &'s str,
) -> Result<FreeFunction<'s>, DemangleError<'s>> {
    let allow_array_fixup = true;

    let (argument_list, static_suffix) = demangle_argument_list(
        config,
        args,
        None,
        &TemplatedValueVec::new(),
        allow_array_fixup,
    )?;

    let ff = FreeFunction::new(func_name, argument_list, static_suffix);
    Ok(ff)
}
