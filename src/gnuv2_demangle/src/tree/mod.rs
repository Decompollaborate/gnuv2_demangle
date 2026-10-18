/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

mod arg;
mod arg_qualifier;
mod arg_vec;
mod custom_name;
mod demangled;
mod destructor;
mod global_sym_keyed;
mod in_function_static_initialized;
mod method_qualifier;
mod namespace;
mod namespaced_global;
mod namespaced_type;
mod static_suffix;
mod template;
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
pub(crate) use static_suffix::StaticSuffix;
pub(crate) use template::Template;
pub(crate) use typ::{Type, TypeMeta};

pub(crate) use global_sym_keyed::{GlobalSymKeyedKind, GlobalSymKeyedSym};

pub use demangled::Demangled;
pub use destructor::Destructor;
pub use global_sym_keyed::GlobalSymKeyed;
pub use in_function_static_initialized::InFunctionStaticInitialized;
pub use namespaced_global::NamespacedGlobal;
pub use type_info::{TypeInfoFunction, TypeInfoNode};
pub use virtual_table::VirtualTable;
