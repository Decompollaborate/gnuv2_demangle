/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::borrow::Cow;

/// The result of partially or totally consuming an str from left to right,
/// storing the part that haven't been consumed yet (`remaining`) and the
/// consumed part (`data`), possibly converted to a different type.
///
/// This can be seen as a way of partial parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub(crate) struct Remaining<'s, T> {
    /// The remaining data that haven't been parsed yet.
    pub(crate) r: &'s str,
    /// The parsed data.
    pub(crate) d: T,
}

impl<'s, T> Remaining<'s, T> {
    pub(crate) const fn new(remaining: &'s str, data: T) -> Self {
        Self {
            r: remaining,
            d: data,
        }
    }
}

impl<'s, 'd> Remaining<'s, &'d str> {
    pub(crate) fn d_as_cow(self) -> Remaining<'s, Cow<'d, str>> {
        let Self { r, d } = self;

        Remaining::new(r, Cow::from(d))
    }
}

impl<'s> Remaining<'s, &'s str> {
    pub(crate) fn split_at(s: &'s str, mid: usize) -> Self {
        let (data, remaining) = s.split_at(mid);

        Self::new(remaining, data)
    }
}

pub(crate) trait StrParsing<'s> {
    #[must_use]
    fn p_number(&'s self) -> Option<Remaining<'s, usize>>;
    #[must_use]
    fn p_hex_number(&'s self) -> Option<Remaining<'s, usize>>;
    #[must_use]
    fn p_digit(&'s self) -> Option<Remaining<'s, usize>>;
    /// Parse either a single digit followed by nondigits or a multidigit followed
    /// by an underscore.
    #[must_use]
    fn p_number_maybe_multi_digit(&'s self) -> Option<Remaining<'s, usize>>;

    #[must_use]
    fn p_first(&'s self) -> Option<Remaining<'s, char>>;
}

impl<'s> StrParsing<'s> for str {
    fn p_number(&'s self) -> Option<Remaining<'s, usize>> {
        let (remaining, data) = if let Some(index) = self.find(|c: char| !c.is_ascii_digit()) {
            (&self[index..], self[..index].parse().ok()?)
        } else {
            ("", self.parse().ok()?)
        };

        Some(Remaining::new(remaining, data))
    }

    fn p_hex_number(&'s self) -> Option<Remaining<'s, usize>> {
        let (remaining, data) = if let Some(index) = self.find(|c: char| !c.is_ascii_hexdigit()) {
            (
                &self[index..],
                usize::from_str_radix(&self[..index], 16).ok()?,
            )
        } else {
            ("", usize::from_str_radix(self, 16).ok()?)
        };

        Some(Remaining::new(remaining, data))
    }

    fn p_digit(&'s self) -> Option<Remaining<'s, usize>> {
        let c = self.chars().next()?;
        if c.is_ascii_digit() {
            let digit = (c as usize).wrapping_sub('0' as usize);

            Some(Remaining::new(&self[1..], digit))
        } else {
            None
        }
    }

    fn p_number_maybe_multi_digit(&'s self) -> Option<Remaining<'s, usize>> {
        if self.is_empty() {
            None
        } else if self.len() == 1 {
            // Single digit should be fine to just parse
            Some(Remaining::new("", self.parse().ok()?))
        } else if let Some(index) = self.find(|c: char| !c.is_ascii_digit()) {
            if index == 0 {
                None
            } else if self[index..].starts_with('_') {
                // Skip the leading underscore only if this is not a single
                // digit value
                let new_start = if index > 1 { index + 1 } else { index };
                Some(Remaining::new(
                    &self[new_start..],
                    self[..index].parse().ok()?,
                ))
            } else {
                // Only consume a single digit
                Some(Remaining::new(&self[1..], self[..1].parse().ok()?))
            }
        } else {
            // Only consume a single digit
            Some(Remaining::new(&self[1..], self[..1].parse().ok()?))
        }
    }

    fn p_first(&'s self) -> Option<Remaining<'s, char>> {
        let c = self.chars().next()?;

        Some(Remaining::new(&self[1..], c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_maybe_multi_digit() {
        assert_eq!(
            "1junk".p_number_maybe_multi_digit(),
            Some(Remaining::new("junk", 1)),
        );
        assert_eq!(
            "12_junk".p_number_maybe_multi_digit(),
            Some(Remaining::new("junk", 12)),
        );
        assert_eq!(
            "54junk".p_number_maybe_multi_digit(),
            Some(Remaining::new("4junk", 5)),
        );
        assert_eq!(
            "2".p_number_maybe_multi_digit(),
            Some(Remaining::new("", 2)),
        );
        assert_eq!(
            "32".p_number_maybe_multi_digit(),
            Some(Remaining::new("2", 3)),
        );
        assert_eq!(
            "1_junk".p_number_maybe_multi_digit(),
            Some(Remaining::new("_junk", 1)),
        );
    }
}
