/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

mod arg;
mod arg_qualifier;
mod arg_vec;
mod custom_name;
mod method_qualifier;
mod namespace;
mod namespaced_type;
mod operator;
mod static_suffix;
mod template;
mod templated_value_vec;
mod typ;

pub(crate) use arg::{ArrayQualifiers, DemangledArg, FunctionPointer, MethodPointer, Signedness};
pub(crate) use arg_qualifier::{ArgPostQualifier, ArgPostQualifierVec, ArgQualifiers};
pub(crate) use arg_vec::ArgVec;
pub(crate) use custom_name::CustomName;
pub(crate) use method_qualifier::MethodQualifier;
pub(crate) use namespace::NamespaceVec;
pub(crate) use namespaced_type::{NamespaceKind, NamespacedType};
pub(crate) use operator::Operator;
pub(crate) use static_suffix::StaticSuffix;
pub(crate) use template::Template;
pub(crate) use templated_value_vec::{
    TemplateElement, TemplatedTypename, TemplatedValue, TemplatedValueVec,
};
pub(crate) use typ::{Type, TypeMeta};
