/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;
use core::fmt;

use super::{CustomName, Template};

use crate::{tree::Demangled, utils};

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

    pub(crate) fn ends_with_gt(&self) -> bool {
        self.namepace_list.last().is_some_and(|x| x.ends_with_gt())
    }

    pub(crate) fn push(&mut self, ns: NamespaceElement<'s>) {
        self.namepace_list.push(ns);
    }
}

impl<'s> fmt::Display for NamespaceVec<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.namepace_list.iter();

        utils::write_joining(f, iter, "::")?;

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

impl<'s> NamespaceElement<'s> {
    pub(crate) fn ends_with_gt(&self) -> bool {
        match self {
            Self::Template(template) => template.ends_with_gt(),
            Self::DemangledSymbol(demangled) => demangled.ends_with_gt(),
            Self::Plain(_custom_name) => false,
        }
    }
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
