/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    borrow::Cow,
    string::{String, ToString},
};
use core::{fmt, num::NonZeroU32};

use super::{ArgPostQualifierVec, ArrayQualifiers, TypeMeta};

use crate::option_display::OptionDisplay;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DemangledArg<'s> {
    Plain(PlainArgArray<'s>),
    FunctionPointer(FunctionPointer),
    MethodPointer(MethodPointer<'s>),
    ObjectPointer(String),
    Repeat { count: NonZeroU32, index: u32 },
    Ellipsis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PlainArgArray<'s> {
    pub(crate) plain: PlainArg<'s>,
    pub(crate) array_qualifiers: Option<ArrayQualifiers>,
}

impl<'s> PlainArgArray<'s> {
    pub(crate) const fn new(
        plain: PlainArg<'s>,
        array_qualifiers: Option<ArrayQualifiers>,
    ) -> Self {
        Self {
            plain,
            array_qualifiers,
        }
    }
}

impl fmt::Display for PlainArgArray<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            plain,
            array_qualifiers,
        } = self;

        write!(f, "{plain}")?;
        if let Some(array_qualifiers) = array_qualifiers {
            write!(f, "{array_qualifiers}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PlainArg<'s> {
    pub(crate) typ: TypeMeta<'s>,
    pub(crate) post_qualifiers: ArgPostQualifierVec,
}

impl<'s> PlainArg<'s> {
    pub(crate) const fn new(typ: TypeMeta<'s>, post_qualifiers: ArgPostQualifierVec) -> Self {
        Self {
            typ,
            post_qualifiers,
        }
    }
}

impl fmt::Display for PlainArg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let PlainArg {
            typ,
            post_qualifiers,
        } = self;

        write!(f, "{typ}")?;
        if !post_qualifiers.is_empty() {
            write!(f, " {}", post_qualifiers.to_string().trim_matches(' '))?;
        }
        Ok(())
    }
}

impl From<PlainArg<'_>> for Cow<'static, str> {
    fn from(value: PlainArg<'_>) -> Self {
        Cow::from(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FunctionPointer {
    pub(crate) return_type: String,
    pub(crate) array_qualifiers: OptionDisplay<ArrayQualifiers>,
    pub(crate) post_qualifiers: String,
    pub(crate) args: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MethodPointer<'s> {
    pub(crate) return_type: String,
    pub(crate) array_qualifiers: OptionDisplay<ArrayQualifiers>,
    pub(crate) class: PlainArg<'s>,
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

impl fmt::Display for MethodPointer<'_> {
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
