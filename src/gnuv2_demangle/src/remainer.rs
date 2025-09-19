/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

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

impl<'s> Remaining<'s, &'s str> {
    pub(crate) fn split_at(s: &'s str, mid: usize) -> Self {
        let (data, remaining) = s.split_at(mid);

        Self::new(remaining, data)
    }
}

pub(crate) trait StrNumParsing<'s> {
    fn p_number(&'s self) -> Option<Remaining<'s, usize>>;
    fn p_digit(&'s self) -> Option<Remaining<'s, usize>>;
    /// Parse either a single digit followed by nondigits or a multidigit followed
    /// by an underscore.
    fn p_number_maybe_multi_digit(&'s self) -> Option<Remaining<'s, usize>>;
}

impl<'s> StrNumParsing<'s> for str {
    fn p_number(&'s self) -> Option<Remaining<'s, usize>> {
        let (remaining, data) = if let Some(index) = self.find(|c: char| !c.is_ascii_digit()) {
            (&self[index..], self[..index].parse().ok()?)
        } else {
            ("", self.parse().ok()?)
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
                // Number can be followed by an underscore only if it is a
                // multidigit value
                if index > 1 {
                    Some(Remaining::new(
                        &self[index + 1..],
                        self[..index].parse().ok()?,
                    ))
                } else {
                    None
                }
            } else {
                // Only consume a single digit
                Some(Remaining::new(&self[1..], self[..1].parse().ok()?))
            }
        } else {
            // Only consume a single digit
            Some(Remaining::new(&self[1..], self[..1].parse().ok()?))
        }
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
        assert_eq!("1_junk".p_number_maybe_multi_digit(), None,);
    }
}
