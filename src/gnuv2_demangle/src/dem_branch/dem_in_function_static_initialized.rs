/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    str_cutter::StrCutter, tree::InFunctionStaticInitialized, DemangleConfig, DemangleError,
};

pub(crate) fn demangle_in_function_static_initialized<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<InFunctionStaticInitialized, DemangleError<'s>> {
    if !config.demangle_in_function_static_initialized {
        return Err(DemangleError::InFunctionStaticInitializedDisabled(s));
    }

    let Some((left, right)) = s.c_split2_char('.') else {
        return Err(DemangleError::InFunctionStaticInitializedMissingPair(s));
    };

    let left = left
        .parse()
        .map_err(|_| DemangleError::InFunctionStaticInitializedFirstNotNumber(left))?;
    let right = right
        .parse()
        .map_err(|_| DemangleError::InFunctionStaticInitializedSecondNotNumber(right))?;

    Ok(InFunctionStaticInitialized::new(left, right))
}
