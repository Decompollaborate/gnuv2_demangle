/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::sync::Arc;
use core::fmt;

use crate::tree::leaf::{ArgVec, MethodQualifier, NamespacedType, Operator, StaticSuffix};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperatorMethod<'s> {
    operator: Operator,
    method_qualifier: MethodQualifier,
    namespace: Arc<NamespacedType<'s>>,
    argument_list: ArgVec<'s>,
    static_suffix: StaticSuffix<'s>,
}

impl<'s> OperatorMethod<'s> {
    pub(crate) const fn new(
        operator: Operator,
        method_qualifier: MethodQualifier,
        namespace: Arc<NamespacedType<'s>>,
        argument_list: ArgVec<'s>,
        static_suffix: StaticSuffix<'s>,
    ) -> Self {
        Self {
            operator,
            method_qualifier,
            namespace,
            argument_list,
            static_suffix,
        }
    }
}

impl fmt::Display for OperatorMethod<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            operator,
            method_qualifier,
            namespace,
            argument_list,
            static_suffix,
        } = self;
        let namespace = &namespace.namespace;

        write!(
            f,
            "{namespace}::{operator}{static_suffix}({argument_list}){method_qualifier}"
        )
    }
}
