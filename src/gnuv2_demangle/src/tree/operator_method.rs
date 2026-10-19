/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperatorMethod<'s> {
    s: String,
    p: core::marker::PhantomData<&'s str>,
}

impl<'s> OperatorMethod<'s> {
    pub(crate) const fn new(s: String) -> Self {
        Self {
            s,
            p: core::marker::PhantomData,
        }
    }
}

impl fmt::Display for OperatorMethod<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { s, p: _ } = self;

        write!(f, "{s}")
    }
}
