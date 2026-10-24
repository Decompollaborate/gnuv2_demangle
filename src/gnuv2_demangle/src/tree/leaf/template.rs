/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;

use alloc::string::String;

use super::{CustomName, NamespaceKind, NamespacedType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Template<'s> {
    template: String,
    class_name: CustomName<'s>,
}

impl<'s> Template<'s> {
    pub(crate) const fn new(template: String, class_name: CustomName<'s>) -> Self {
        Self {
            template,
            class_name,
        }
    }

    pub(crate) fn class_name(&self) -> CustomName<'s> {
        self.class_name
    }

    pub(crate) fn into_namespaced_type(self) -> NamespacedType<'s> {
        let class_name = self.class_name;

        NamespacedType::new(NamespaceKind::Template(self), class_name)
    }
}

impl fmt::Display for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            template,
            class_name,
        } = self;

        if template.ends_with('>') {
            write!(f, "{}<{} >", class_name, template)
        } else {
            write!(f, "{}<{}>", class_name, template)
        }
    }
}
