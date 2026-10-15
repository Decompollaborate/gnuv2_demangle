/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::borrow::Cow;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NamespaceVec<'s> {
    namepace_list: Vec<Cow<'s, str>>,
}

impl<'s> NamespaceVec<'s> {
    pub(crate) const fn new() -> Self {
        Self {
            namepace_list: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, ns: Cow<'s, str>) {
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

// silly
impl<'s> From<&NamespaceVec<'s>> for Cow<'s, str> {
    fn from(value: &NamespaceVec<'s>) -> Self {
        if value.namepace_list.len() == 1 {
            Cow::clone(
                value
                    .namepace_list
                    .first()
                    .expect("Already checked the len"),
            )
        } else {
            Cow::from(value.to_string())
        }
    }
}
impl<'s> From<NamespaceVec<'s>> for Cow<'s, str> {
    fn from(value: NamespaceVec<'s>) -> Self {
        if value.namepace_list.len() == 1 {
            value
                .namepace_list
                .into_iter()
                .next()
                .expect("Already checked the len")
        } else {
            Cow::from(value.to_string())
        }
    }
}
