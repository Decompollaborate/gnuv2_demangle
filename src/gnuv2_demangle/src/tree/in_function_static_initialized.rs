/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InFunctionStaticInitialized {
    left: u32,
    right: u32,
}

impl InFunctionStaticInitialized {
    pub(crate) const fn new(left: u32, right: u32) -> Self {
        Self { left, right }
    }
}

impl fmt::Display for InFunctionStaticInitialized {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { left, right } = self;

        // made up convention
        write!(f, "in-function-static initialized {left} {right}")
    }
}
