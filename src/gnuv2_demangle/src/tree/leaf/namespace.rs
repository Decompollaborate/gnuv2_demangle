/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;
use core::fmt;

use super::{CustomName, Template};

use crate::tree::Demangled;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NamespaceVec<'s> {
    namepace_list: Vec<NamespaceElement<'s>>,
}

impl<'s> NamespaceVec<'s> {
    pub(crate) const fn new() -> Self {
        Self {
            namepace_list: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, ns: NamespaceElement<'s>) {
        self.namepace_list.push(ns);
    }
}

impl<'s> fmt::Display for NamespaceVec<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.namepace_list.iter();

        let Some(n) = iter.next() else {
            return Ok(());
        };

        write!(f, "{n}")?;
        for n in iter {
            write!(f, "::{n}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[expect(clippy::large_enum_variant)]
pub(crate) enum NamespaceElement<'s> {
    Template(Template<'s>),
    DemangledSymbol(Demangled<'s>),
    Plain(CustomName<'s>),
}

impl fmt::Display for NamespaceElement<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Template(template) => write!(f, "{template}"),
            Self::DemangledSymbol(demangled) => write!(f, "{demangled}"),
            Self::Plain(custom_name) => write!(f, "{custom_name}"),
        }
    }
}
