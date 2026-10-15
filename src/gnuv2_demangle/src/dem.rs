/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use alloc::borrow::Cow;

use crate::DemangleError;

use crate::remainer::{Remaining, StrParsing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CustomName<'s> {
    name: &'s str,
}

impl<'s> CustomName<'s> {
    pub(crate) const fn new(name: &'s str) -> Self {
        Self { name }
    }

    pub(crate) const fn into_raw(self) -> &'s str {
        self.name
    }
}

impl<'s> fmt::Display for CustomName<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// silly
impl<'s> From<CustomName<'s>> for Cow<'s, str> {
    fn from(value: CustomName<'s>) -> Self {
        Cow::from(value.name)
    }
}

pub(crate) fn demangle_custom_name<'s, F>(
    s: &'s str,
    err: F,
) -> Result<Remaining<'s, CustomName<'s>>, DemangleError<'s>>
where
    F: Fn(&'s str) -> DemangleError<'s>,
{
    let Remaining { r, d: length } = s.p_number().ok_or_else(|| err(s))?;

    if r.len() < length {
        Err(err(s))
    } else {
        Ok(Remaining::split_at(r, length).convert_data(CustomName::new))
    }
}

pub(crate) fn demangle_method_qualifier(s: &str) -> Remaining<'_, &str> {
    if let Some(remaining) = s.strip_prefix('C') {
        Remaining::new(remaining, " const")
    } else {
        Remaining::new(s, "")
    }
}
