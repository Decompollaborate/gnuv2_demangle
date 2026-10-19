/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    borrow::Cow,
    string::{String, ToString},
};
use core::fmt;

use super::{
    Destructor, FreeFunction, GlobalSymKeyed, InFunctionStaticInitialized, Method,
    NamespacedFunction, NamespacedGlobal, TypeInfoFunction, TypeInfoNode, VirtualTable,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Demangled<'s> {
    Destructor(Destructor<'s>),
    TypeInfoFunction(TypeInfoFunction<'s>),
    TypeInfoNode(TypeInfoNode<'s>),
    VirtualTable(VirtualTable<'s>),
    NamespacedTypeClassMethod(String),
    OperatorFree(String),
    OperatorMethod(String),
    GlobalSymKeyed(GlobalSymKeyed<'s>),
    InFunctionStaticInitialized(InFunctionStaticInitialized),
    FreeFunction(FreeFunction<'s>),
    Method(Method<'s>), // ClassFunction :thinking:
    TemplatedFunction(String),
    NamespacedFunction(NamespacedFunction<'s>),
    NamespacedGlobal(NamespacedGlobal<'s>),
}

impl<'s> Demangled<'s> {
    // TODO: remove when we get rid of every String variant.
    pub(crate) fn into_string(self) -> String {
        match self {
            Self::Destructor(inner) => inner.to_string(),
            Self::TypeInfoFunction(inner) => inner.to_string(),
            Self::TypeInfoNode(inner) => inner.to_string(),
            Self::VirtualTable(inner) => inner.to_string(),
            Self::NamespacedTypeClassMethod(inner) => inner,
            Self::OperatorFree(inner) => inner,
            Self::OperatorMethod(inner) => inner,
            Self::GlobalSymKeyed(inner) => inner.to_string(),
            Self::InFunctionStaticInitialized(inner) => inner.to_string(),
            Self::FreeFunction(inner) => inner.to_string(),
            Self::Method(inner) => inner.to_string(),
            Self::TemplatedFunction(inner) => inner,
            Self::NamespacedFunction(inner) => inner.to_string(),
            Self::NamespacedGlobal(inner) => inner.to_string(),
        }
    }
}

impl<'s> fmt::Display for Demangled<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Destructor(inner) => write!(f, "{inner}"),
            Self::TypeInfoFunction(inner) => write!(f, "{inner}"),
            Self::TypeInfoNode(inner) => write!(f, "{inner}"),
            Self::VirtualTable(inner) => write!(f, "{inner}"),
            Self::NamespacedTypeClassMethod(inner) => write!(f, "{inner}"),
            Self::OperatorFree(inner) => write!(f, "{inner}"),
            Self::OperatorMethod(inner) => write!(f, "{inner}"),
            Self::GlobalSymKeyed(inner) => write!(f, "{inner}"),
            Self::InFunctionStaticInitialized(inner) => write!(f, "{inner}"),
            Self::FreeFunction(inner) => write!(f, "{inner}"),
            Self::Method(inner) => write!(f, "{inner}"),
            Self::TemplatedFunction(inner) => write!(f, "{inner}"),
            Self::NamespacedFunction(inner) => write!(f, "{inner}"),
            Self::NamespacedGlobal(inner) => write!(f, "{inner}"),
        }
    }
}

// silly
impl<'s> From<Demangled<'s>> for Cow<'s, str> {
    fn from(value: Demangled) -> Self {
        Cow::from(value.into_string())
    }
}
