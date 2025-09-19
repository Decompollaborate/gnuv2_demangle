/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

mod demangler;
mod demangle_config;
mod demangle_error;

pub use demangle_config::DemangleConfig;
pub use demangle_error::DemangleError;
pub use demangler::demangle;
