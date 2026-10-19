/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

mod arg;
mod arg_qualifier;
mod arg_vec;
mod custom_name;
mod demangled;
mod destructor;
mod free_function;
mod global_sym_keyed;
mod in_function_static_initialized;
mod method;
mod method_qualifier;
mod namespace;
mod namespaced_function;
mod namespaced_global;
mod namespaced_type;
mod namespaced_type_class_method;
mod operator;
mod operator_function;
mod operator_method;
mod static_suffix;
mod template;
mod templated_function;
mod typ;
mod type_info;
mod virtual_table;

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
pub(crate) use typ::{Type, TypeMeta};

pub(crate) use global_sym_keyed::{GlobalSymKeyedKind, GlobalSymKeyedSym};

pub use demangled::Demangled;
pub use destructor::Destructor;
pub use free_function::FreeFunction;
pub use global_sym_keyed::GlobalSymKeyed;
pub use in_function_static_initialized::InFunctionStaticInitialized;
pub use method::Method;
pub use namespaced_function::NamespacedFunction;
pub use namespaced_global::NamespacedGlobal;
pub use namespaced_type_class_method::NamespacedTypeClassMethod;
pub use operator_function::OperatorFunction;
pub use operator_method::OperatorMethod;
pub use templated_function::TemplatedFunction;
pub use type_info::{TypeInfoFunction, TypeInfoNode};
pub use virtual_table::VirtualTable;
