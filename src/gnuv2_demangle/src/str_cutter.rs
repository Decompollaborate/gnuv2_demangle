/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

pub(crate) trait StrCutter<'s> {
    #[must_use]
    fn c_split2(&'s self, pat: &str) -> Option<(&'s str, &'s str)>;
    #[must_use]
    fn c_split2_r_starts_with<F>(&'s self, pat: &str, r_cond: F) -> Option<(&'s str, &'s str)>
    where
        F: Fn(char) -> bool;

    #[must_use]
    fn c_cond_and_strip_prefix(&'s self, cond: bool, prefix: &str) -> Option<&'s str>;

    #[must_use]
    fn c_maybe_strip_prefix(&'s self, c: char) -> (&'s str, bool);
}

impl<'s> StrCutter<'s> for str {
    fn c_split2(&'s self, pat: &str) -> Option<(&'s str, &'s str)> {
        let mut iter = self.splitn(2, pat);

        if let (Some(l), Some(r)) = (iter.next(), iter.next()) {
            if l.is_empty() || r.is_empty() {
                None
            } else {
                Some((l, r))
            }
        } else {
            None
        }
    }

    fn c_split2_r_starts_with<F>(&'s self, pat: &str, r_cond: F) -> Option<(&'s str, &'s str)>
    where
        F: Fn(char) -> bool,
    {
        let mut iter = self.splitn(2, pat);

        if let (Some(l), Some(r)) = (iter.next(), iter.next()) {
            if l.is_empty() {
                None
            } else if r.starts_with(r_cond) {
                Some((l, r))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn c_cond_and_strip_prefix(&'s self, cond: bool, prefix: &str) -> Option<&'s str> {
        if cond {
            self.strip_prefix(prefix)
        } else {
            None
        }
    }

    fn c_maybe_strip_prefix(&'s self, c: char) -> (&'s str, bool) {
        if let Some(a) = self.strip_prefix(c) {
            (a, true)
        } else {
            (self, false)
        }
    }
}
