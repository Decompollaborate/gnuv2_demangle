/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use crate::tree::leaf::{CustomName, NamespaceKind, StaticSuffix};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Destructor<'s> {
    namespace: NamespaceKind<'s>,
    type_name: CustomName<'s>,
    static_suffix: StaticSuffix<'s>,
}

impl<'s> Destructor<'s> {
    pub(crate) const fn new(
        namespace: NamespaceKind<'s>,
        type_name: CustomName<'s>,
        static_suffix: StaticSuffix<'s>,
    ) -> Self {
        Self {
            namespace,
            type_name,
            static_suffix,
        }
    }
}

impl fmt::Display for Destructor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            namespace,
            type_name,
            static_suffix,
        } = self;

        write!(
            f,
            "{namespace}{static_suffix}::~{type_name}{static_suffix}(void)"
        )
    }
}
