/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;
use core::fmt;

use super::{ArrayQualifiers, Signedness};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgQualifiers {
    pub(crate) sign: Signedness,
    pub(crate) post_qualifiers: ArgPostQualifierVec,
    pub(crate) pre_qualifiers: ArgPreQualifierVec,
    pub(crate) array_qualifiers: Option<ArrayQualifiers>,
}

impl ArgQualifiers {
    pub(crate) const fn new(
        sign: Signedness,
        post_qualifiers: ArgPostQualifierVec,
        pre_qualifiers: ArgPreQualifierVec,
        array_qualifiers: Option<ArrayQualifiers>,
    ) -> Self {
        Self {
            sign,
            post_qualifiers,
            pre_qualifiers,
            array_qualifiers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ArgPostQualifier {
    Pointer,
    Reference,
    Const,
    Volatile,
}

// Qualifiers should be displayed in reverse order
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgPostQualifierVec {
    qualifiers: Vec<ArgPostQualifier>,
}

impl ArgPostQualifierVec {
    pub(crate) const fn new() -> Self {
        Self {
            qualifiers: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, qualifier: ArgPostQualifier) {
        self.qualifiers.push(qualifier);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.qualifiers.is_empty()
    }

    pub(crate) fn ends_with_ptr_like(&self) -> bool {
        matches!(
            self.qualifiers.first(),
            Some(ArgPostQualifier::Pointer | ArgPostQualifier::Reference)
        )
    }

    pub(crate) fn valid_as_method_pointer_arg(&self) -> bool {
        // method pointer args have a few restrictions on how they are qualified.

        // starting from the last one
        let mut iter = self.qualifiers.iter().rev();

        // Last must be a pointer
        if iter.next() != Some(&ArgPostQualifier::Pointer) {
            return false;
        }

        // const are bad
        // volatile are fine, but not two in a row
        // only one reference is allowed, and it can only be the first element
        let mut saw_reference = false;
        let mut prev = None;
        for q in iter {
            if saw_reference {
                return false;
            }

            match q {
                ArgPostQualifier::Pointer => {}
                ArgPostQualifier::Reference => {
                    saw_reference = true;
                }
                ArgPostQualifier::Const => return false,
                ArgPostQualifier::Volatile => {
                    if prev == Some(*q) {
                        return false;
                    }
                }
            }

            prev = Some(*q);
        }

        true
    }
}

impl fmt::Display for ArgPostQualifierVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut needs_space = false;

        for q in self.qualifiers.iter().rev() {
            if needs_space {
                write!(f, " ")?;
                needs_space = false;
            }

            match q {
                ArgPostQualifier::Pointer => write!(f, "*")?,
                ArgPostQualifier::Reference => write!(f, "&")?,
                ArgPostQualifier::Const => {
                    write!(f, "const")?;
                    needs_space = true;
                }
                ArgPostQualifier::Volatile => {
                    write!(f, "volatile")?;
                    needs_space = true;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ArgPreQualifier {
    Const,
    Volatile,
}

impl ArgPreQualifier {
    pub(crate) const fn from_char(c: char) -> Option<Self> {
        match c {
            'C' => Some(Self::Const),
            'V' => Some(Self::Volatile),
            _ => None,
        }
    }
}

impl fmt::Display for ArgPreQualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Const => write!(f, "const"),
            Self::Volatile => write!(f, "volatile"),
        }
    }
}

// Qualifiers should be displayed in reverse order
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgPreQualifierVec {
    qualifiers: [Option<ArgPreQualifier>; 2],
    count: u8,
}

impl ArgPreQualifierVec {
    pub(crate) const fn new() -> Self {
        Self {
            qualifiers: [None; 2],
            count: 0,
        }
    }

    pub(crate) fn push(&mut self, qualifier: ArgPreQualifier) -> bool {
        let count = self.count as usize;

        // avoid duplicated entries
        if self.qualifiers[0] == Some(qualifier) || self.qualifiers[1] == Some(qualifier) {
            return false;
        }

        if count < self.qualifiers.len() {
            self.qualifiers[count] = Some(qualifier);
            self.count += 1;
            true
        } else {
            false
        }
    }
}

impl fmt::Display for ArgPreQualifierVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let back_skip = self.qualifiers.len().saturating_sub(self.count as usize);

        for q in self.qualifiers.iter().rev().skip(back_skip) {
            let Some(q) = q else {
                break;
            };

            write!(f, "{q} ")?;
        }

        Ok(())
    }
}
