/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CustomName<'s> {
    name: &'s str,
}

impl<'s> CustomName<'s> {
    pub(crate) const fn new(name: &'s str) -> Self {
        Self { name }
    }

    pub(crate) const fn into_raw(self) -> &'s str {
        self.name
    }
}

impl<'s> fmt::Display for CustomName<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
