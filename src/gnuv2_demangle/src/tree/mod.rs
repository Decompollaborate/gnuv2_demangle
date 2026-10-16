/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

mod arg;
mod arg_vec;
mod custom_name;
mod method_qualifier;
mod namespace;
mod namespaced_type;
mod static_suffix;
mod template;

pub(crate) use arg::{ArrayQualifiers, DemangledArg, FunctionPointer, MethodPointer, Signedness};
pub(crate) use arg_vec::ArgVec;
pub(crate) use custom_name::CustomName;
pub(crate) use method_qualifier::MethodQualifier;
pub(crate) use namespace::NamespaceVec;
pub(crate) use namespaced_type::{NamespaceKind, NamespacedType};
pub(crate) use static_suffix::StaticSuffix;
pub(crate) use template::Template;
