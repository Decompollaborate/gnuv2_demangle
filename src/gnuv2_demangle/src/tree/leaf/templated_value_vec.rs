/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use super::{CustomName, FunctionPointer, MethodPointer, PlainArgArray};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TemplatedValueVec<'s> {
    vec: Vec<TemplateElement<'s>>,
}

impl<'s> TemplatedValueVec<'s> {
    pub(crate) const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub(crate) fn get(&self, index: usize) -> Option<String> {
        self.vec.get(index).map(ToString::to_string)
    }

    pub(crate) fn push(&mut self, element: TemplateElement<'s>) {
        self.vec.push(element);
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
    FunctionPointer(FunctionPointer),
    MethodPointer(MethodPointer),
    ObjectPointer(String),
    #[expect(dead_code)]
    Dummy(core::marker::PhantomData<&'s str>),
}

impl fmt::Display for TemplatedTypename<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plain(p) => write!(f, "{p}"),
            Self::FunctionPointer(function_pointer) => {
                write!(f, "{function_pointer}")
            }
            Self::MethodPointer(method_pointer) => write!(f, "{method_pointer}"),
            Self::ObjectPointer(op) => write!(f, "{op}"),
            Self::Dummy(_phantom_data) => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemplatedValue<'s> {
    ObjectReference(String),
    FunctionPointer(String),
    FunctionReference(String),
    FunctionRaw(String),
    Character(char),
    LookbackInt(String),
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
