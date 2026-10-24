/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

pub(crate) fn write_joining<T, I>(
    f: &mut fmt::Formatter<'_>,
    mut iter: T,
    glue: &str,
) -> Result<bool, fmt::Error>
where
    T: Iterator<Item = I>,
    I: fmt::Display,
{
    let mut wrote_anything = false;

    if let Some(first) = iter.next() {
        write!(f, "{first}")?;
        wrote_anything = true;
    }

    for arg in iter {
        write!(f, "{glue}{arg}")?;
        wrote_anything = true;
    }

    Ok(wrote_anything)
}
