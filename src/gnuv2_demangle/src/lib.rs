/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

mod demangle_config;
mod demangle_error;
pub(crate) mod demangler;

pub use demangle_config::DemangleConfig;
pub use demangle_error::DemangleError;
pub use demangler::demangle;

// internal utilities
pub(crate) mod dem;
pub(crate) mod dem_arg;
pub(crate) mod dem_arg_list;
pub(crate) mod dem_arg_qualifiers;
pub(crate) mod dem_namespace;
pub(crate) mod dem_namespaced_type;
pub(crate) mod dem_other;
pub(crate) mod dem_template;

pub(crate) mod tree;

pub(crate) mod option_display;
pub(crate) mod remainer;
pub(crate) mod str_cutter;
