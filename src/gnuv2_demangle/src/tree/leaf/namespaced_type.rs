/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use super::{CustomName, NamespaceVec, Template};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NamespacedType<'s> {
    pub(crate) namespace: NamespaceKind<'s>,
    pub(crate) type_name: CustomName<'s>,
}

impl<'s> NamespacedType<'s> {
    pub(crate) const fn new(namespace: NamespaceKind<'s>, type_name: CustomName<'s>) -> Self {
        Self {
            namespace,
            type_name,
        }
    }

    pub(crate) fn ends_with_gt(&self) -> bool {
        self.namespace.ends_with_gt()
    }
}

impl<'s> fmt::Display for NamespacedType<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            namespace,
            type_name: _,
        } = self;

        write!(f, "{namespace}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum NamespaceKind<'s> {
    Template(Template<'s>),
    Namespace(NamespaceVec<'s>),
    Class(CustomName<'s>),
}

impl<'s> NamespaceKind<'s> {
    pub(crate) fn ends_with_gt(&self) -> bool {
        match self {
            Self::Template(template) => template.ends_with_gt(),
            Self::Namespace(namespace_vec) => namespace_vec.ends_with_gt(),
            Self::Class(_custom_name) => false,
        }
    }
}

impl<'s> fmt::Display for NamespaceKind<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Template(t) => write!(f, "{t}"),
            Self::Namespace(ns) => write!(f, "{ns}"),
            Self::Class(c) => write!(f, "{c}"),
        }
    }
}
