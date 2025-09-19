/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::DemangleError;

use crate::remainer::{Remaining, StrParsing};

pub(crate) fn demangle_custom_name<'s, F>(
    s: &'s str,
    err: F,
) -> Result<Remaining<'s, &'s str>, DemangleError<'s>>
where
    F: Fn(&'s str) -> DemangleError<'s>,
{
    let Remaining { r, d: length } = s.p_number().ok_or_else(|| err(s))?;

    if r.len() < length {
        Err(err(s))
    } else {
        Ok(Remaining::split_at(r, length))
    }
}

pub(crate) fn demangle_method_qualifier(s: &str) -> Remaining<'_, &str> {
    if let Some(remaining) = s.strip_prefix('C') {
        Remaining::new(remaining, " const")
    } else {
        Remaining::new(s, "")
    }
}
