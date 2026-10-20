/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, sync::Arc};
use core::fmt;

use super::{NamespacedType, StaticSuffix};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespacedFunction<'s> {
    function_name: &'s str,
    argument_list: Cow<'s, str>,
    namespace: Arc<NamespacedType<'s>>,
    static_suffix: StaticSuffix<'s>,
}

impl<'s> NamespacedFunction<'s> {
    pub(crate) const fn new(
        function_name: &'s str,
        argument_list: Cow<'s, str>,
        namespace: Arc<NamespacedType<'s>>,
        static_suffix: StaticSuffix<'s>,
    ) -> Self {
        Self {
            function_name,
            argument_list,
            namespace,
            static_suffix,
        }
    }
}

impl<'s> fmt::Display for NamespacedFunction<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            function_name,
            argument_list,
            namespace,
            static_suffix,
        } = self;

        write!(
            f,
            "{namespace}::{function_name}{static_suffix}({argument_list})"
        )
    }
}
