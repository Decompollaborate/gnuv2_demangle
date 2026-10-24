/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;
use core::fmt;

use crate::{tree::leaf::NamespacedType, utils};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtualTable<'s> {
    namespaces: Vec<NamespacedType<'s>>,
}

impl<'s> VirtualTable<'s> {
    pub(crate) const fn new(namespaces: Vec<NamespacedType<'s>>) -> Self {
        Self { namespaces }
    }
}

impl fmt::Display for VirtualTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { namespaces } = self;

        let iter = namespaces.iter();
        utils::write_joining(f, iter, "::")?;

        write!(f, " virtual table")
    }
}
