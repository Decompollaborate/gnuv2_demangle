/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use super::{
    Destructor, FreeFunction, GlobalSymKeyed, InFunctionStaticInitialized, Method,
    NamespacedFunction, NamespacedGlobal, NamespacedTypeClassMethod, OperatorFunction,
    OperatorMethod, TemplatedFunction, TypeInfoFunction, TypeInfoNode, VirtualTable,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Demangled<'s> {
    Destructor(Destructor<'s>),
    TypeInfoFunction(TypeInfoFunction<'s>),
    TypeInfoNode(TypeInfoNode<'s>),
    VirtualTable(VirtualTable<'s>),
    NamespacedTypeClassMethod(NamespacedTypeClassMethod<'s>),
    OperatorFunction(OperatorFunction<'s>),
    OperatorMethod(OperatorMethod<'s>),
    GlobalSymKeyed(GlobalSymKeyed<'s>),
    InFunctionStaticInitialized(InFunctionStaticInitialized),
    FreeFunction(FreeFunction<'s>),
    Method(Method<'s>), // ClassFunction :thinking:
    TemplatedFunction(TemplatedFunction<'s>),
    NamespacedFunction(NamespacedFunction<'s>),
    NamespacedGlobal(NamespacedGlobal<'s>),
}

impl<'s> Demangled<'s> {
    pub(crate) fn ends_with_gt(&self) -> bool {
        false
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
            Self::OperatorFunction(inner) => write!(f, "{inner}"),
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
