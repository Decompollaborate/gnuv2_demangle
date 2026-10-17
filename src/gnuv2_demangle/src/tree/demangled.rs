/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, string::String};
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Demangled<'s> {
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

    // TODO: remove when any other variant depends on lifetime 's
    #[doc(hidden)]
    Dummy(core::marker::PhantomData<&'s str>)
}

impl<'s> Demangled<'s> {
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
            Self::Dummy(_inner) => unreachable!(),
        }
    }
}

impl<'s> fmt::Display for Demangled<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Destructor(inner) => write!(f, "{inner}"),
            Self::TypeInfoFunction(inner) => write!(f, "{inner}"),
            Self::TypeInfoNode(inner) => write!(f, "{inner}"),
            Self::NamespacedType(inner) => write!(f, "{inner}"),
            Self::OperatorFree(inner) => write!(f, "{inner}"),
            Self::OperatorMethod(inner) => write!(f, "{inner}"),
            Self::GlobalSymKeyed(inner) => write!(f, "{inner}"),
            Self::InFunctionStaticInitialized(inner) => write!(f, "{inner}"),
            Self::FreeFunction(inner) => write!(f, "{inner}"),
            Self::Method(inner) => write!(f, "{inner}"),
            Self::TemplatedFunction(inner) => write!(f, "{inner}"),
            Self::NamespacedFunction(inner) => write!(f, "{inner}"),
            Self::VirtualTable(inner) => write!(f, "{inner}"),
            Self::NamespacedGlobal(inner) => write!(f, "{inner}"),
            Self::Dummy(_inner) => unreachable!(),
        }
    }
}

// silly
impl<'s> From<Demangled<'s>> for Cow<'s, str> {
    fn from(value: Demangled) -> Self {
        Cow::from(value.into_string())
    }
}
