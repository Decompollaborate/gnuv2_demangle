/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

pub(crate) trait StrCutter<'s> {
    #[must_use]
    fn c_split2(&'s self, pat: &str) -> Option<(&'s str, &'s str)>;
    #[must_use]
    fn c_split2_char(&'s self, pat: char) -> Option<(&'s str, &'s str)>;
    #[must_use]
    fn c_split2_r_starts_with<F>(
        &'s self,
        pat: &str,
        r_cond: F,
    ) -> Option<(&'s str, &'s str, char)>
    where
        F: Fn(char) -> bool;

    #[must_use]
    fn c_cond_and_strip_prefix_and_char(
        &'s self,
        cond: bool,
        prefix: &str,
        c: char,
    ) -> Option<&'s str>;

    #[must_use]
    fn c_maybe_strip_prefix(&'s self, c: char) -> (&'s str, bool);

    #[must_use]
    fn c_strip_prefix_3chars(&'s self, a: char, b: char, c: char) -> Option<&'s str>;
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

    fn c_split2_char(&'s self, pat: char) -> Option<(&'s str, &'s str)> {
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

    fn c_split2_r_starts_with<F>(&'s self, pat: &str, r_cond: F) -> Option<(&'s str, &'s str, char)>
    where
        F: Fn(char) -> bool,
    {
        // This assumes ASCII

        // Start at index 1 to avoid an empty `left`.
        for i in 1..self.len() {
            let current = &self[i..];

            // If current is smaller than the pattern then there's no point
            // in continue looking.
            if current.len() <= pat.len() {
                break;
            }

            // Kinda like an `split`
            if let Some(right) = current.strip_prefix(pat) {
                if right.starts_with(&r_cond) {
                    let left = &self[..i];
                    let first_right_character =
                        right
                            .chars()
                            .next()
                            .expect("Due to the previous start_with we expect this to have at least a single character");

                    return Some((left, right, first_right_character));
                }
            }
        }

        None
    }

    fn c_cond_and_strip_prefix_and_char(
        &'s self,
        cond: bool,
        prefix: &str,
        c: char,
    ) -> Option<&'s str> {
        if cond {
            self.strip_prefix(prefix).and_then(|x| x.strip_prefix(c))
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

    fn c_strip_prefix_3chars(&'s self, a: char, b: char, c: char) -> Option<&'s str> {
        self.strip_prefix(a)
            .and_then(|x| x.strip_prefix(b).and_then(|y| y.strip_prefix(c)))
    }
}
