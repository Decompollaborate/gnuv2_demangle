/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::borrow::Cow;
use core::fmt;

use super::StaticSuffix;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FreeFunction<'s> {
    function_name: &'s str,
    argument_list: Cow<'s, str>,
    static_suffix: StaticSuffix<'s>,
}

impl<'s> FreeFunction<'s> {
    pub(crate) const fn new(
        function_name: &'s str,
        argument_list: Cow<'s, str>,
        static_suffix: StaticSuffix<'s>,
    ) -> Self {
        Self {
            function_name,
            argument_list,
            static_suffix,
        }
    }
}

impl<'s> fmt::Display for FreeFunction<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            function_name,
            argument_list,
            static_suffix,
        } = self;

        write!(f, "{function_name}{static_suffix}({argument_list})")
    }
}
