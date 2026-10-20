/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;
use core::fmt;

use super::{ArrayQualifiers, Signedness};

use crate::option_display::OptionDisplay;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgQualifiers {
    pub(crate) sign: Signedness,
    pub(crate) post_qualifiers: ArgPostQualifierVec,
    pub(crate) array_qualifiers: OptionDisplay<ArrayQualifiers>,
}

impl ArgQualifiers {
    pub(crate) const fn new(
        sign: Signedness,
        post_qualifiers: ArgPostQualifierVec,
        array_qualifiers: OptionDisplay<ArrayQualifiers>,
    ) -> Self {
        Self {
            sign,
            post_qualifiers,
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
        for q in self.qualifiers.iter().rev() {
            match q {
                ArgPostQualifier::Pointer => write!(f, "*")?,
                ArgPostQualifier::Reference => write!(f, "&")?,
                ArgPostQualifier::Const => write!(f, "const ")?,
                ArgPostQualifier::Volatile => write!(f, "volatile ")?,
            }
        }

        Ok(())
    }
}
