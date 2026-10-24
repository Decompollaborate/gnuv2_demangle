/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, sync::Arc};
use core::fmt;

use super::{CustomName, NamespacedType, Signedness, Template, TemplateElement};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TypeMeta<'s> {
    sign: Signedness,
    typ: Type<'s>,
}

impl<'s> TypeMeta<'s> {
    pub(crate) const fn new(sign: Signedness, typ: Type<'s>) -> Self {
        Self { sign, typ }
    }

    pub(crate) const fn class_like(&self) -> bool {
        self.typ.class_like()
    }
}

impl fmt::Display for TypeMeta<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { sign, typ } = self;

        write!(f, "{sign}{typ}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Type<'s> {
    /// 'c'
    Char,
    /// 's'
    Short,
    /// 'i'
    Int,
    /// 'l'
    Long,
    /// 'x'
    LongLong,
    /// 'f'
    Float,
    /// 'd'
    Double,
    /// 'r'
    LongDouble,
    /// 'b'
    Bool,
    /// 'w'
    WChar,
    /// 'v'
    Void,
    /// 'I'
    UInt128,
    /// 'I'
    SInt128,
    /// 'I'
    Int128,
    /// '1'..='9'
    Class(CustomName<'s>),
    /// 'Q'
    Namespaces(NamespacedType<'s>),
    /// 'T'
    LookBack {
        referenced_arg: Cow<'s, str>,
        lookback: u32,
    },
    /// 't'
    Template(Template<'s>),
    /// 'X'
    TemplatedFunctionIndex {
        referenced_arg: Arc<TemplateElement<'s>>,
        lookback: u32,
    },
}

impl<'s> Type<'s> {
    pub(crate) const fn class_like(&self) -> bool {
        match self {
            Self::Char => false,
            Self::Short => false,
            Self::Int => false,
            Self::Long => false,
            Self::LongLong => false,
            Self::Float => false,
            Self::Double => false,
            Self::LongDouble => false,
            Self::Bool => false,
            Self::WChar => false,
            Self::Void => false,
            Self::UInt128 => false,
            Self::SInt128 => false,
            Self::Int128 => false,
            Self::Class(_) => true,
            Self::Namespaces(..) => true,
            Self::LookBack { .. } => false,
            Self::Template(_) => true,
            Self::TemplatedFunctionIndex { .. } => false,
        }
    }
}

impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Char => write!(f, "char"),
            Type::Short => write!(f, "short"),
            Type::Int => write!(f, "int"),
            Type::Long => write!(f, "long"),
            Type::LongLong => write!(f, "long long"),
            Type::Float => write!(f, "float"),
            Type::Double => write!(f, "double"),
            Type::LongDouble => write!(f, "long double"),
            Type::Bool => write!(f, "bool"),
            Type::WChar => write!(f, "wchar_t"),
            Type::Void => write!(f, "void"),
            Type::UInt128 => write!(f, "__uint128_t"),
            Type::SInt128 => write!(f, "__int128_t"),
            Type::Int128 => write!(f, "int128_t"),
            Type::Class(class_name) => write!(f, "{class_name}"),
            Type::Namespaces(namespace_vec) => write!(f, "{namespace_vec}"),
            Type::LookBack {
                referenced_arg,
                lookback: _,
            } => write!(f, "{referenced_arg}"),
            Type::Template(template) => write!(f, "{template}"),
            Type::TemplatedFunctionIndex {
                referenced_arg,
                lookback: _,
            } => write!(f, "{referenced_arg}"),
        }
    }
}
