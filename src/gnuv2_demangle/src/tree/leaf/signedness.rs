/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Signedness {
    No,
    Signed,
    Unsigned,
}

impl Signedness {
    pub(crate) const fn from_char(c: char) -> Option<Self> {
        match c {
            'S' => Some(Self::Signed),
            'U' => Some(Self::Unsigned),
            _ => None,
        }
    }
}

impl fmt::Display for Signedness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => Ok(()),
            Self::Signed => write!(f, "signed "),
            Self::Unsigned => write!(f, "unsigned "),
        }
    }
}
