/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::string::String;
use core::fmt;

use crate::{option_display::OptionDisplay, tree::leaf::ArrayQualifiers};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeInfoFunction<'s> {
    typ: String,
    array_qualifiers: OptionDisplay<ArrayQualifiers>,

    // TODO: remove
    _force_lifetime: core::marker::PhantomData<&'s str>,
}

impl<'s> TypeInfoFunction<'s> {
    pub(crate) const fn new(typ: String, array_qualifiers: OptionDisplay<ArrayQualifiers>) -> Self {
        Self {
            typ,
            array_qualifiers,
            _force_lifetime: core::marker::PhantomData,
        }
    }
}

impl<'s> fmt::Display for TypeInfoFunction<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            typ,
            array_qualifiers,
            _force_lifetime: _,
        } = self;

        write!(f, "{typ}{array_qualifiers} type_info function")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeInfoNode<'s> {
    typ: String,
    array_qualifiers: OptionDisplay<ArrayQualifiers>,

    // TODO: remove
    _force_lifetime: core::marker::PhantomData<&'s str>,
}

impl<'s> TypeInfoNode<'s> {
    pub(crate) const fn new(typ: String, array_qualifiers: OptionDisplay<ArrayQualifiers>) -> Self {
        Self {
            typ,
            array_qualifiers,
            _force_lifetime: core::marker::PhantomData,
        }
    }
}

impl<'s> fmt::Display for TypeInfoNode<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            typ,
            array_qualifiers,
            _force_lifetime: _,
        } = self;

        write!(f, "{typ}{array_qualifiers} type_info node")
    }
}
