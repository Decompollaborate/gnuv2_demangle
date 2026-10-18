/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use super::NamespacedType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespacedGlobal<'s> {
    namespace: NamespacedType<'s>,
    name: &'s str,
}

impl<'s> NamespacedGlobal<'s> {
    pub(crate) const fn new(namespace: NamespacedType<'s>, name: &'s str) -> Self {
        Self { namespace, name }
    }
}

impl fmt::Display for NamespacedGlobal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { namespace, name } = self;

        let space = &namespace.namespace;

        write!(f, "{space}::{name}")
    }
}
