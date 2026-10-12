/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use crate::{
    remainer::{Remaining, StrParsing},
    DemangleConfig, DemangleError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StaticSuffix<'s> {
    s: Option<&'s str>,
}

impl<'s> StaticSuffix<'s> {
    pub(crate) const fn none() -> Self {
        Self { s: None }
    }
    pub(crate) const fn new(s: &'s str) -> Self {
        Self { s: Some(s) }
    }
}

impl<'s> fmt::Display for StaticSuffix<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.s {
            write!(f, ".{}", s)
        } else {
            Ok(())
        }
    }
}

pub(crate) fn demangle_static_suffix<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<Remaining<'s, StaticSuffix<'s>>, DemangleError<'s>> {
    if !config.demangle_static_suffix {
        // Treat this function as a no-op if this setting is disabled.
        // We could consider making this a proper error instead, but idk if
        // it is worth.
        return Ok(Remaining::new(s, StaticSuffix::none()));
    }

    // Handle the static numeric suffix used to distinguish static symbols inside structs
    let r = if let Some(extra) = s.strip_prefix('.') {
        let Some(rem) = extra.p_number_str() else {
            return Err(DemangleError::NonNumericStaticSuffix(s));
        };

        Remaining::new(rem.r, StaticSuffix::new(rem.d))
    } else {
        Remaining::new(s, StaticSuffix::none())
    };

    Ok(r)
}
