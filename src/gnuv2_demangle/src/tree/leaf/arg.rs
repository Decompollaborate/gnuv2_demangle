/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{boxed::Box, string::String};
use core::{fmt, num::NonZeroU32};

use super::{ArgPostQualifierVec, ArgVec, ArrayQualifiers, TypeMeta};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DemangledArg<'s> {
    Plain(PlainArgArray<'s>),
    FunctionPointer(FunctionPointer<'s>),
    MethodPointer(MethodPointer<'s>),
    MemberPointer(MemberPointer<'s>),
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

impl<'s> PlainArgArray<'s> {
    pub(crate) fn ends_with_gt(&self) -> bool {
        if self.array_qualifiers.is_some() {
            false
        } else {
            self.plain.ends_with_gt()
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

    fn ends_with_ptr_like(&self) -> bool {
        self.post_qualifiers.ends_with_ptr_like()
    }

    pub(crate) fn ends_with_gt(&self) -> bool {
        if self.post_qualifiers.is_empty() {
            self.typ.ends_with_gt()
        } else {
            false
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
            write!(f, " {post_qualifiers}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FunctionPointer<'s> {
    pub(crate) return_type: FunctionPointerReturnType<'s>,
    pub(crate) array_qualifiers: Option<ArrayQualifiers>,
    pub(crate) post_qualifiers: String,
    pub(crate) args: ArgVec<'s>,
}

impl<'s> FunctionPointer<'s> {
    pub(crate) const fn new(
        return_type: FunctionPointerReturnType<'s>,
        array_qualifiers: Option<ArrayQualifiers>,
        post_qualifiers: String,
        args: ArgVec<'s>,
    ) -> Self {
        Self {
            return_type,
            array_qualifiers,
            post_qualifiers,
            args,
        }
    }

    fn ends_with_ptr_like(&self) -> bool {
        true
    }
}

impl fmt::Display for FunctionPointer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FunctionPointer {
            return_type,
            array_qualifiers,
            post_qualifiers,
            args,
        } = self;

        let mut wrote_space = false;

        // We can't just write! the return_type, since it may be a function
        // pointer, and those wrap the argument around instead of just being at
        // the beginning of the declaration.
        return_type.fmt_prev(f)?;

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
        if !return_type.ends_with_ptr_like() && !wrote_space {
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

        // Display the rest of the returned value, mostly for function pointers.
        return_type.fmt_post(f)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MethodPointer<'s> {
    pub(crate) return_type: FunctionPointerReturnType<'s>,
    pub(crate) array_qualifiers: Option<ArrayQualifiers>,
    pub(crate) class: PlainArg<'s>,
    pub(crate) post_qualifiers: String,
    pub(crate) args: ArgVec<'s>,
    pub(crate) is_const_method: bool,
}

impl<'s> MethodPointer<'s> {
    pub(crate) const fn new(
        return_type: FunctionPointerReturnType<'s>,
        array_qualifiers: Option<ArrayQualifiers>,
        class: PlainArg<'s>,
        post_qualifiers: String,
        args: ArgVec<'s>,
        is_const_method: bool,
    ) -> Self {
        Self {
            return_type,
            array_qualifiers,
            class,
            post_qualifiers,
            args,
            is_const_method,
        }
    }

    fn ends_with_ptr_like(&self) -> bool {
        true
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

        let mut wrote_space = false;

        return_type.fmt_prev(f)?;

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
        if !return_type.ends_with_ptr_like() && !wrote_space {
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

        return_type.fmt_post(f)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MemberPointer<'s> {
    class_name: PlainArg<'s>,
    member_type: PlainArg<'s>,
    array_qualifiers: Option<ArrayQualifiers>,
    post_qualifiers: ArgPostQualifierVec,
}

impl<'s> MemberPointer<'s> {
    pub(crate) const fn new(
        class_name: PlainArg<'s>,
        member_type: PlainArg<'s>,
        array_qualifiers: Option<ArrayQualifiers>,
        post_qualifiers: ArgPostQualifierVec,
    ) -> Self {
        Self {
            class_name,
            member_type,
            array_qualifiers,
            post_qualifiers,
        }
    }
}

impl fmt::Display for MemberPointer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            class_name,
            member_type,
            array_qualifiers,
            post_qualifiers,
        } = self;

        // Arrays makes everything harder.
        write!(f, "{member_type} ")?;

        if let Some(arr) = &array_qualifiers {
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, "({}", arr.inner_post_qualifiers)?;
            }
        }

        write!(f, "({class_name}::{post_qualifiers})")?;
        if let Some(arr) = &array_qualifiers {
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, ")")?;
            }
            write!(f, "{}", arr.arrays)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum FunctionPointerReturnType<'s> {
    PlainArg(PlainArg<'s>),
    FunctionPointer(Box<FunctionPointer<'s>>),
    MethodPointer(Box<MethodPointer<'s>>),
    MemberPointer(MemberPointer<'s>),
}

impl<'s> FunctionPointerReturnType<'s> {
    fn ends_with_ptr_like(&self) -> bool {
        match self {
            Self::PlainArg(plain) => plain.ends_with_ptr_like(),
            Self::FunctionPointer(function_pointer) => function_pointer.ends_with_ptr_like(),
            Self::MethodPointer(method_pointer) => method_pointer.ends_with_ptr_like(),
            Self::MemberPointer(..) => true,
        }
    }

    pub(crate) fn fmt_prev(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionPointerReturnType::PlainArg(plain_arg) => write!(f, "{plain_arg}"),
            FunctionPointerReturnType::FunctionPointer(function_pointer) => {
                let FunctionPointer {
                    return_type,
                    array_qualifiers: _,
                    post_qualifiers,
                    args: _,
                } = function_pointer.as_ref();

                return_type.fmt_prev(f)?;
                write!(f, " (")?;
                write!(f, "{post_qualifiers}")
            }
            FunctionPointerReturnType::MethodPointer(method_pointer) => {
                let MethodPointer {
                    return_type: _,
                    array_qualifiers: _,
                    class,
                    post_qualifiers,
                    args: _,
                    is_const_method: _,
                } = method_pointer.as_ref();

                method_pointer.return_type.fmt_prev(f)?;
                write!(f, " (")?;
                write!(f, "{class}::{post_qualifiers}")
            }
            FunctionPointerReturnType::MemberPointer(member_pointer) => {
                write!(f, "{member_pointer}")
            }
        }
    }

    pub(crate) fn fmt_post(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionPointerReturnType::PlainArg(..) => {}
            FunctionPointerReturnType::FunctionPointer(function_pointer) => {
                let FunctionPointer {
                    return_type: _,
                    array_qualifiers,
                    post_qualifiers: _,
                    args,
                } = function_pointer.as_ref();

                write!(f, ")")?;
                write!(f, "({args})")?;
                if let Some(array_qualifiers) = array_qualifiers {
                    write!(f, "{array_qualifiers}")?;
                }
            }
            FunctionPointerReturnType::MethodPointer(method_pointer) => {
                let MethodPointer {
                    return_type: _,
                    array_qualifiers,
                    class: _,
                    post_qualifiers: _,
                    args,
                    is_const_method,
                } = method_pointer.as_ref();

                write!(f, ")")?;
                write!(f, "({args})")?;
                if *is_const_method {
                    write!(f, " const")?;
                }
                if let Some(array_qualifiers) = array_qualifiers {
                    write!(f, "{array_qualifiers}")?;
                }
            }
            FunctionPointerReturnType::MemberPointer(..) => {}
        }

        Ok(())
    }
}

impl<'s> From<PlainArg<'s>> for FunctionPointerReturnType<'s> {
    fn from(value: PlainArg<'s>) -> Self {
        Self::PlainArg(value)
    }
}
impl<'s> From<FunctionPointer<'s>> for FunctionPointerReturnType<'s> {
    fn from(value: FunctionPointer<'s>) -> Self {
        Self::FunctionPointer(Box::new(value))
    }
}
impl<'s> From<MethodPointer<'s>> for FunctionPointerReturnType<'s> {
    fn from(value: MethodPointer<'s>) -> Self {
        Self::MethodPointer(Box::new(value))
    }
}
impl<'s> From<MemberPointer<'s>> for FunctionPointerReturnType<'s> {
    fn from(value: MemberPointer<'s>) -> Self {
        Self::MemberPointer(value)
    }
}
