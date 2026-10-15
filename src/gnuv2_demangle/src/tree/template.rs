/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::string::String;

use super::{CustomName, NamespaceKind, NamespacedType};

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

    pub(crate) fn into_template(self) -> String {
        self.template
    }
    pub(crate) fn class_name(&self) -> CustomName<'s> {
        self.class_name
    }

    pub(crate) fn into_namespaced_type(self) -> NamespacedType<'s> {
        let Self {
            template,
            class_name,
        } = self;

        NamespacedType::new(NamespaceKind::Template(template), class_name)
    }
}
