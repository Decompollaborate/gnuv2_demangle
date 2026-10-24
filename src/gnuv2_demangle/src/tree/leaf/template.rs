/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use super::{CustomName, NamespaceKind, NamespacedType, TemplatedValueVec};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Template<'s> {
    class_name: CustomName<'s>,
    template: TemplatedValueVec<'s>,
}

impl<'s> Template<'s> {
    pub(crate) const fn new(class_name: CustomName<'s>, template: TemplatedValueVec<'s>) -> Self {
        Self {
            class_name,
            template,
        }
    }

    pub(crate) fn class_name(&self) -> CustomName<'s> {
        self.class_name
    }

    pub(crate) fn ends_with_gt(&self) -> bool {
        true
    }

    pub(crate) fn into_namespaced_type(self) -> NamespacedType<'s> {
        let class_name = self.class_name;

        NamespacedType::new(NamespaceKind::Template(self), class_name)
    }
}

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            class_name,
            template,
        } = self;

        if template.ends_with_gt() {
            write!(f, "{}<{} >", class_name, template)
        } else {
            write!(f, "{}<{}>", class_name, template)
        }
    }
}
