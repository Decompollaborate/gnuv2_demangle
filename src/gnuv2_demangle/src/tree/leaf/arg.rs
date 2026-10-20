/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::string::String;
use core::{fmt, num::NonZeroUsize};

use super::ArgPostQualifierVec;

use crate::option_display::OptionDisplay;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DemangledArg {
    Plain(String, OptionDisplay<ArrayQualifiers>),
    FunctionPointer(FunctionPointer),
    MethodPointer(MethodPointer),
    ObjectPointer(String),
    Repeat { count: NonZeroUsize, index: usize },
    Ellipsis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FunctionPointer {
    pub(crate) return_type: String,
    pub(crate) array_qualifiers: OptionDisplay<ArrayQualifiers>,
    pub(crate) post_qualifiers: String,
    pub(crate) args: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MethodPointer {
    pub(crate) return_type: String,
    pub(crate) array_qualifiers: OptionDisplay<ArrayQualifiers>,
    pub(crate) class: String, // TODO: `&'s str` instead? should be easy, i think...
    pub(crate) post_qualifiers: String,
    pub(crate) args: String,
    pub(crate) is_const_method: bool,
}

impl fmt::Display for FunctionPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FunctionPointer {
            return_type,
            array_qualifiers,
            post_qualifiers,
            args,
        } = self;

        let array_qualifiers = array_qualifiers.as_option();
        let mut wrote_space = false;

        write!(f, "{return_type}")?;
        if let Some(arr) = array_qualifiers {
            // A return type being a pointer to array has a terrible syntax,
            // which we need to break up.
            // Instead of writing `int (*)[3] (*)(void)` (function pointer
            // returning a pointer to array), we have to instead write
            // `int (*(*)(void))[3]`.

            write!(f, " ")?;
            wrote_space = true;
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, "({}", arr.inner_post_qualifiers)?;
            }
        }
        if !return_type.ends_with(['*', '&']) && !wrote_space {
            write!(f, " ")?;
        }
        write!(f, "({})", post_qualifiers.trim_matches(' '))?;
        write!(f, "({args})")?;
        if let Some(arr) = array_qualifiers {
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, ")")?;
            }
            write!(f, "{}", arr.arrays)?;
        }
        Ok(())
    }
}

impl fmt::Display for MethodPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MethodPointer {
            return_type,
            array_qualifiers,
            class,
            post_qualifiers,
            args,
            is_const_method,
        } = self;

        let array_qualifiers = array_qualifiers.as_option();
        let mut wrote_space = false;

        write!(f, "{return_type}")?;
        if let Some(arr) = array_qualifiers {
            // A return type being a pointer to array has a terrible syntax,
            // which we need to break up.
            // Instead of writing `int (*)[3] (*)(void)` (function pointer
            // returning a pointer to array), we have to instead write
            // `int (*(*)(void))[3]`.

            write!(f, " ")?;
            wrote_space = true;
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, "({}", arr.inner_post_qualifiers)?;
            }
        }
        if !return_type.ends_with(['*', '&']) && !wrote_space {
            write!(f, " ")?;
        }
        write!(f, "({}::{})", class, post_qualifiers.trim_matches(' '))?;
        write!(f, "({args})")?;
        if *is_const_method {
            write!(f, " const")?;
        }
        if let Some(arr) = array_qualifiers {
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, ")")?;
            }
            write!(f, "{}", arr.arrays)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Signedness {
    No,
    Signed,
    Unsigned,
}

impl fmt::Display for Signedness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => Ok(()),
            Self::Signed => write!(f, "signed "),
            Self::Unsigned => write!(f, "unsigned "),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArrayQualifiers {
    pub(crate) inner_post_qualifiers: ArgPostQualifierVec,
    // TODO: would be better to store this as a vector instead of an array?
    pub(crate) arrays: String,
}

impl fmt::Display for ArrayQualifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")?;

        if !self.inner_post_qualifiers.is_empty() {
            // Only add parenthesis if there are post_qualifiers, like a
            // pointer.
            // Arrays without being decaying to pointers can happen in, for
            // example, templated functions.
            write!(f, "({})", self.inner_post_qualifiers)?;
        }

        write!(f, "{}", self.arrays)
    }
}
