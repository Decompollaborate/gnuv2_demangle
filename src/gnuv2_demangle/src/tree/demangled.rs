/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, string::String};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Demangled {
    Destructor(String),
    TypeInfoFunction(String),
    TypeInfoNode(String),
    NamespacedType(String),
    OperatorFree(String),
    OperatorMethod(String),
    GlobalSymKeyed(String),
    InFunctionStaticInitialized(String),
    FreeFunction(String),
    Method(String), // ClassFunction :thinking:
    TemplatedFunction(String),
    NamespacedFunction(String),
    VirtualTable(String),
    NamespacedGlobal(String),
}

impl Demangled {
    // TODO: remove when we get rid of every String variant.
    pub(crate) fn into_string(self) -> String {
        match self {
            Self::Destructor(inner) => inner,
            Self::TypeInfoFunction(inner) => inner,
            Self::TypeInfoNode(inner) => inner,
            Self::NamespacedType(inner) => inner,
            Self::OperatorFree(inner) => inner,
            Self::OperatorMethod(inner) => inner,
            Self::GlobalSymKeyed(inner) => inner,
            Self::InFunctionStaticInitialized(inner) => inner,
            Self::FreeFunction(inner) => inner,
            Self::Method(inner) => inner,
            Self::TemplatedFunction(inner) => inner,
            Self::NamespacedFunction(inner) => inner,
            Self::VirtualTable(inner) => inner,
            Self::NamespacedGlobal(inner) => inner,
        }
    }
}

impl From<Demangled> for Cow<'_, str> {
    fn from(value: Demangled) -> Self {
        Cow::from(value.into_string())
    }
}
