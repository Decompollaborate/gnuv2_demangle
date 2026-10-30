/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;
use core::fmt;

use super::{ArgPostQualifierVec, ArgPreQualifierVec};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArrayQualifiers {
    pub(crate) inner_post_qualifiers: ArgPostQualifierVec,
    pub(crate) inner_pre_qualifiers: ArgPreQualifierVec,
    pub(crate) arrays: ArrayVec,
}

impl ArrayQualifiers {
    pub(crate) const fn new(
        inner_post_qualifiers: ArgPostQualifierVec,
        inner_pre_qualifiers: ArgPreQualifierVec,
        arrays: ArrayVec,
    ) -> Self {
        Self {
            inner_post_qualifiers,
            inner_pre_qualifiers,
            arrays,
        }
    }
}

impl fmt::Display for ArrayQualifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            inner_post_qualifiers,
            inner_pre_qualifiers,
            arrays,
        } = self;

        write!(f, " {inner_pre_qualifiers}")?;

        if !inner_post_qualifiers.is_empty() {
            // Only add parenthesis if there are post_qualifiers, like a
            // pointer.
            // Arrays without being decaying to pointers can happen in, for
            // example, templated functions.
            write!(f, "({})", self.inner_post_qualifiers)?;
        }

        write!(f, "{arrays}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArrayVec {
    arrays: Vec<u32>,
}

impl ArrayVec {
    pub(crate) const fn new(arrays: Vec<u32>) -> Self {
        Self { arrays }
    }
}

impl fmt::Display for ArrayVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { arrays } = self;

        for arr_length in arrays {
            write!(f, "[{arr_length}]")?;
        }
        Ok(())
    }
}
