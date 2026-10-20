/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::borrow::Cow;
use core::fmt;

use super::{CustomName, NamespaceKind, StaticSuffix};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespacedTypeClassMethod<'s> {
    method_name: CustomName<'s>,
    namespace: NamespaceKind<'s>,
    argument_list: Cow<'s, str>,
    static_suffix: StaticSuffix<'s>,
}

impl<'s> NamespacedTypeClassMethod<'s> {
    pub(crate) const fn new(
        method_name: CustomName<'s>,
        namespace: NamespaceKind<'s>,
        argument_list: Cow<'s, str>,
        static_suffix: StaticSuffix<'s>,
    ) -> Self {
        Self {
            method_name,
            namespace,
            argument_list,
            static_suffix,
        }
    }
}

impl fmt::Display for NamespacedTypeClassMethod<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            method_name,
            namespace,
            argument_list,
            static_suffix,
        } = self;

        write!(
            f,
            "{namespace}::{method_name}{static_suffix}({argument_list})"
        )
    }
}
