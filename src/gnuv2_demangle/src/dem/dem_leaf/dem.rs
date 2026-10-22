/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    remainer::{Remaining, StrParsing},
    tree::leaf::{CustomName, MethodQualifier},
    DemangleError,
};

pub(crate) fn demangle_custom_name<'s, F>(
    s: &'s str,
    err: F,
) -> Result<Remaining<'s, CustomName<'s>>, DemangleError<'s>>
where
    F: Fn(&'s str) -> DemangleError<'s>,
{
    let Remaining { r, d: length } = s.p_number().ok_or_else(|| err(s))?;

    if r.len() < length as usize {
        Err(err(s))
    } else {
        Ok(Remaining::split_at(r, length as usize).convert_data(CustomName::new))
    }
}

pub(crate) fn demangle_method_qualifier<'s>(s: &'s str) -> Remaining<'s, MethodQualifier> {
    if let Some(remaining) = s.strip_prefix('C') {
        Remaining::new(remaining, MethodQualifier::Const)
    } else {
        Remaining::new(s, MethodQualifier::No)
    }
}
