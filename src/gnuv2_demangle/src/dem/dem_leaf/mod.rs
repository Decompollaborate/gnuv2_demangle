/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

mod dem;
mod dem_arg;
mod dem_arg_list;
mod dem_arg_qualifiers;
mod dem_namespace;
mod dem_namespaced_type;
mod dem_operator;
mod dem_other;
mod dem_template;

pub(crate) use dem::{demangle_custom_name, demangle_method_qualifier};
pub(crate) use dem_arg::demangle_argument;
pub(crate) use dem_arg_list::{demangle_argument_list, demangle_argument_list_impl};
pub(crate) use dem_arg_qualifiers::demangle_arg_qualifiers;
pub(crate) use dem_namespace::demangle_namespaces;
pub(crate) use dem_namespaced_type::demangle_namespaced_type;
pub(crate) use dem_operator::demangle_operator;
pub(crate) use dem_other::demangle_static_suffix;
pub(crate) use dem_template::{demangle_template, demangle_template_with_return_type};
