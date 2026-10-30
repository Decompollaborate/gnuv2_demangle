/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

mod arg;
mod arg_qualifier;
mod arg_vec;
mod array;
mod custom_name;
mod method_qualifier;
mod namespace;
mod namespaced_type;
mod operator;
mod signedness;
mod static_suffix;
mod template;
mod templated_value_vec;
mod typ;

pub(crate) use arg::{
    DemangledArg, FunctionPointer, FunctionPointerReturnType, MemberPointer, MethodPointer,
    PlainArg, PlainArgArray,
};
pub(crate) use arg_qualifier::{
    ArgPostQualifier, ArgPostQualifierVec, ArgPreQualifier, ArgPreQualifierVec, ArgQualifiers,
};
pub(crate) use arg_vec::{ArgVec, ResolvedProcessedArg};
pub(crate) use array::{ArrayQualifiers, ArrayVec};
pub(crate) use custom_name::CustomName;
pub(crate) use method_qualifier::MethodQualifier;
pub(crate) use namespace::{NamespaceElement, NamespaceVec};
pub(crate) use namespaced_type::{NamespaceKind, NamespacedType};
pub(crate) use operator::Operator;
pub(crate) use signedness::Signedness;
pub(crate) use static_suffix::StaticSuffix;
pub(crate) use template::Template;
pub(crate) use templated_value_vec::{
    FunctionPointerValue, FunctionRawValue, FunctionReferenceValue, ObjectReferenceValue,
    TemplateElement, TemplatedTypename, TemplatedValue, TemplatedValueVec,
};
pub(crate) use typ::{Type, TypeMeta};
