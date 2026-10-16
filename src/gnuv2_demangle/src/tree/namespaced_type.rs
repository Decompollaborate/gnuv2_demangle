/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, string::String};
use core::fmt;

use super::{CustomName, NamespaceVec};

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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum NamespaceKind<'s> {
    Template(String),
    Namespace(NamespaceVec<'s>),
    Class(CustomName<'s>),
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

// silly
impl<'s> From<&NamespaceKind<'s>> for Cow<'s, str> {
    fn from(value: &NamespaceKind<'s>) -> Self {
        match value {
            NamespaceKind::Template(template) => Cow::from(template.clone()),
            NamespaceKind::Namespace(namespace_vec) => Cow::from(namespace_vec),
            NamespaceKind::Class(class) => Cow::from(*class),
        }
    }
}
