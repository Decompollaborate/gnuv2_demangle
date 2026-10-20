/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::boxed::Box;
use core::fmt;

use super::Demangled;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalSymKeyed<'s> {
    which: GlobalSymKeyedKind,
    sym: GlobalSymKeyedSym<'s>,
}

impl<'s> GlobalSymKeyed<'s> {
    pub(crate) const fn new(which: GlobalSymKeyedKind, sym: GlobalSymKeyedSym<'s>) -> Self {
        Self { which, sym }
    }
}

impl fmt::Display for GlobalSymKeyed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { which, sym } = self;

        write!(f, "global {which} keyed to {sym}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GlobalSymKeyedKind {
    Constructors,
    Destructors,
    Frames,
}

impl GlobalSymKeyedKind {
    pub(crate) const fn is_constructor(&self) -> bool {
        matches!(self, GlobalSymKeyedKind::Constructors)
    }
}

impl fmt::Display for GlobalSymKeyedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Constructors => write!(f, "constructors"),
            Self::Destructors => write!(f, "destructors"),
            Self::Frames => write!(f, "frames"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GlobalSymKeyedSym<'s> {
    Demangled(Box<Demangled<'s>>),
    Raw(&'s str),
}

impl fmt::Display for GlobalSymKeyedSym<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Demangled(demangled) => write!(f, "{demangled}"),
            Self::Raw(raw) => write!(f, "{raw}"),
        }
    }
}
