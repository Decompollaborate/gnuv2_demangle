/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use crate::tree::leaf::{ArgVec, Operator, StaticSuffix};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperatorFunction<'s> {
    operator: Operator<'s>,
    argument_list: ArgVec<'s>,
    static_suffix: StaticSuffix<'s>,
}

impl<'s> OperatorFunction<'s> {
    pub(crate) const fn new(
        operator: Operator<'s>,
        argument_list: ArgVec<'s>,
        static_suffix: StaticSuffix<'s>,
    ) -> Self {
        Self {
            operator,
            argument_list,
            static_suffix,
        }
    }
}

impl fmt::Display for OperatorFunction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            operator,
            argument_list,
            static_suffix,
        } = self;

        write!(f, "{operator}{static_suffix}({argument_list})")
    }
}
