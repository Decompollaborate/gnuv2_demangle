/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::fmt;

use super::{
    ArgVec, CustomName, FunctionPointer, FunctionPointerReturnType, MemberPointer, MethodPointer,
    PlainArgArray,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TemplatedValueVec<'s> {
    vec: Vec<Arc<TemplateElement<'s>>>,
}

impl<'s> TemplatedValueVec<'s> {
    pub(crate) const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub(crate) fn get(&self, index: usize) -> Option<Arc<TemplateElement<'s>>> {
        self.vec.get(index).map(Arc::clone)
    }

    pub(crate) fn push(&mut self, element: TemplateElement<'s>) {
        self.vec.push(Arc::new(element));
    }

    pub(crate) fn join(self) -> String {
        let vec: Vec<String> = self.vec.into_iter().map(|x| x.to_string()).collect();

        vec.join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemplateElement<'s> {
    Typename(TemplatedTypename<'s>), // i.e. class
    Value(TemplatedValue<'s>),
}

impl fmt::Display for TemplateElement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Typename(demangled_arg) => write!(f, "{demangled_arg}"),
            Self::Value(templated_value) => write!(f, "{templated_value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemplatedTypename<'s> {
    Plain(PlainArgArray<'s>),
    FunctionPointer(FunctionPointer<'s>),
    MethodPointer(MethodPointer<'s>),
    MemberPointer(MemberPointer<'s>),
}

impl fmt::Display for TemplatedTypename<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(p) => write!(f, "{p}"),
            Self::FunctionPointer(function_pointer) => {
                write!(f, "{function_pointer}")
            }
            Self::MethodPointer(method_pointer) => write!(f, "{method_pointer}"),
            Self::MemberPointer(op) => write!(f, "{op}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemplatedValue<'s> {
    ObjectReference(ObjectReferenceValue<'s>),
    FunctionPointer(FunctionPointerValue<'s>),
    FunctionReference(FunctionReferenceValue<'s>),
    FunctionRaw(FunctionRawValue<'s>),
    Character(char),
    LookbackInt(Arc<TemplateElement<'s>>),
    Int {
        number: u32,
        negative: bool,
    },
    Bool(bool),
    Enum {
        enum_name: CustomName<'s>,
        number: u32,
        negative: bool,
    },
}

impl fmt::Display for TemplatedValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ObjectReference(s) => write!(f, "{s}"),
            Self::FunctionPointer(s) => write!(f, "{s}"),
            Self::FunctionReference(s) => write!(f, "{s}"),
            Self::FunctionRaw(s) => write!(f, "{s}"),
            Self::Character(c) => write!(f, "'{c}'"),
            Self::LookbackInt(resolved_processed_arg) => write!(f, "{resolved_processed_arg}"),
            Self::Int { number, negative } => {
                if *negative {
                    write!(f, "-")?;
                }
                write!(f, "{number}")
            }
            Self::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Self::Enum {
                enum_name: _,
                number,
                negative,
            } => {
                // TODO: <(SomeEnum)0> is valid c++, try to use it somehow.

                if *negative {
                    write!(f, "-")?;
                }
                write!(f, "{number}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ObjectReferenceValue<'s> {
    symbol: CustomName<'s>,
    is_pointer: bool,
}

impl<'s> ObjectReferenceValue<'s> {
    pub(crate) const fn new(symbol: CustomName<'s>, is_pointer: bool) -> Self {
        Self { symbol, is_pointer }
    }
}

impl fmt::Display for ObjectReferenceValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { symbol, is_pointer } = self;

        if *is_pointer {
            write!(f, "&{symbol}")
        } else {
            write!(f, "{symbol}")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FunctionPointerValue<'s> {
    actual_sym: &'s str,
    args: ArgVec<'s>,
    is_pointer: bool,
    return_type: FunctionPointerReturnType<'s>,
}

impl<'s> FunctionPointerValue<'s> {
    pub(crate) const fn new(
        actual_sym: &'s str,
        args: ArgVec<'s>,
        is_pointer: bool,
        return_type: FunctionPointerReturnType<'s>,
    ) -> Self {
        Self {
            actual_sym,
            args,
            is_pointer,
            return_type,
        }
    }
}

impl fmt::Display for FunctionPointerValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            actual_sym,
            args,
            is_pointer,
            return_type,
        } = self;

        let ampersand = if *is_pointer { "&" } else { "" };
        write!(f, "({return_type}(*)({args})) {ampersand}{actual_sym}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FunctionReferenceValue<'s> {
    actual_sym: &'s str,
    args: ArgVec<'s>,
    is_pointer: bool,
    return_type: FunctionPointerReturnType<'s>,
}

impl<'s> FunctionReferenceValue<'s> {
    pub(crate) const fn new(
        actual_sym: &'s str,
        args: ArgVec<'s>,
        is_pointer: bool,
        return_type: FunctionPointerReturnType<'s>,
    ) -> Self {
        Self {
            actual_sym,
            args,
            is_pointer,
            return_type,
        }
    }
}

impl fmt::Display for FunctionReferenceValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            actual_sym,
            args,
            is_pointer,
            return_type,
        } = self;

        let ampersand = if *is_pointer { "&" } else { "" };
        write!(f, "({return_type}(&)({args})) {ampersand}{actual_sym}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FunctionRawValue<'s> {
    actual_sym: &'s str,
    args: ArgVec<'s>,
    is_pointer: bool,
}

impl<'s> FunctionRawValue<'s> {
    pub(crate) const fn new(actual_sym: &'s str, args: ArgVec<'s>, is_pointer: bool) -> Self {
        Self {
            actual_sym,
            args,
            is_pointer,
        }
    }
}

impl fmt::Display for FunctionRawValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            actual_sym,
            args,
            is_pointer,
        } = self;

        let ampersand = if *is_pointer { "&" } else { "" };
        write!(f, "{ampersand}{actual_sym}({args})")
    }
}
