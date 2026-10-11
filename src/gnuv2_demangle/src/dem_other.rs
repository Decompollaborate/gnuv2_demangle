/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    remainer::{Remaining, StrParsing},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_static_suffix<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<Remaining<'s, &'s str>, DemangleError<'s>> {
    if !config.demangle_static_suffix {
        // Treat this function as a no-op if this setting is disabled.
        // We could consider making this a proper error instead, but idk if
        // it is worth.
        return Ok(Remaining::new(s, ""));
    }

    // Handle the static numeric suffix used to distinguish static symbols inside structs
    let r = if let Some(extra) = s.strip_prefix('.') {
        let Some(rem) = extra.p_number_str() else {
            return Err(DemangleError::NonNumericStaticSuffix(s));
        };

        if !rem.r.is_empty() {
            return Err(DemangleError::TrailingDataAfterStaticSuffix(rem.r, rem.d));
        }

        Remaining::new(rem.r, s)
    } else {
        Remaining::new(s, "")
    };

    Ok(r)
}
