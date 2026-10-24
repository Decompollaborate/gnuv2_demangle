/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

/// An [`Option`] that implements [`Display`] if the inner type does it too.
///
/// The `Display` implemenation invokes the `Display` implemenation from `T` if
/// the variant is `Some`, and does nothing at all if the variant is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub(crate) struct OptionDisplay<T>(Option<T>);

impl<T> fmt::Display for OptionDisplay<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(t) => t.fmt(f),
            // Write nothing on the None case
            None => Ok(()),
        }
    }
}

impl<T> From<Option<T>> for OptionDisplay<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}
