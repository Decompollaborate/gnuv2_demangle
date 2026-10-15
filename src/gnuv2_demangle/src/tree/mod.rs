/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

mod custom_name;
mod namespace;
mod namespaced_type;
mod template;

pub(crate) use custom_name::CustomName;
pub(crate) use namespace::NamespaceVec;
pub(crate) use namespaced_type::{NamespaceKind, NamespacedType};
pub(crate) use template::Template;
