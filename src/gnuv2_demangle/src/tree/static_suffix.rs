/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

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
