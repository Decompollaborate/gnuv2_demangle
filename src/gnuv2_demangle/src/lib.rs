/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unnameable_types)]
#![warn(private_interfaces)]

#[macro_use]
extern crate alloc;

mod demangle_config;
mod demangle_error;
pub(crate) mod demangler;

pub use demangle_config::DemangleConfig;
pub use demangle_error::DemangleError;
pub use demangler::demangle;

// internal utilities
pub(crate) mod dem_branch;
pub(crate) mod dem_leaf;

pub mod tree;

pub(crate) mod option_display;
pub(crate) mod remainer;
pub(crate) mod str_cutter;
