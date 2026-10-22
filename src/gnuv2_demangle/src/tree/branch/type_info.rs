/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use crate::tree::leaf::PlainArgArray;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeInfoFunction<'s> {
    typ: PlainArgArray<'s>,
}

impl<'s> TypeInfoFunction<'s> {
    pub(crate) const fn new(typ: PlainArgArray<'s>) -> Self {
        Self { typ }
    }
}

impl<'s> fmt::Display for TypeInfoFunction<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { typ } = self;

        write!(f, "{typ} type_info function")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeInfoNode<'s> {
    typ: PlainArgArray<'s>,
}

impl<'s> TypeInfoNode<'s> {
    pub(crate) const fn new(typ: PlainArgArray<'s>) -> Self {
        Self { typ }
    }
}

impl<'s> fmt::Display for TypeInfoNode<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { typ } = self;

        write!(f, "{typ} type_info node")
    }
}
