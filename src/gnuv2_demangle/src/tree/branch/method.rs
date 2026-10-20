/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::sync::Arc;
use core::fmt;

use crate::tree::leaf::{ArgVec, MethodQualifier, NamespacedType, StaticSuffix};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Method<'s> {
    method_name: &'s str,
    argument_list: ArgVec<'s>,
    method_qualifier: MethodQualifier,
    namespace: Arc<NamespacedType<'s>>,
    static_suffix: StaticSuffix<'s>,
}

impl<'s> Method<'s> {
    pub(crate) const fn new(
        method_name: &'s str,
        argument_list: ArgVec<'s>,
        method_qualifier: MethodQualifier,
        namespace: Arc<NamespacedType<'s>>,
        static_suffix: StaticSuffix<'s>,
    ) -> Self {
        Self {
            method_name,
            argument_list,
            method_qualifier,
            namespace,
            static_suffix,
        }
    }
}

impl<'s> fmt::Display for Method<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            method_name,
            argument_list,
            method_qualifier,
            namespace,
            static_suffix,
        } = self;
        let namespace = &namespace.namespace;

        write!(
            f,
            "{namespace}::{method_name}{static_suffix}({argument_list}){method_qualifier}"
        )
    }
}
