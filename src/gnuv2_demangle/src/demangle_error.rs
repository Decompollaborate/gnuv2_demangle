/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::{error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DemangleError<'s> {
    NonAscii,
    Invalid,
    TrailingData,
    InvalidClassName(&'s str),
    UnknownType(char),
    RanOutWhileDemanglingSpecial,
    RanOutOfArguments,
    InvalidSpecialMethod(&'s str),
    UnrecognizedSpecialMethod(&'s str),
    PrimitiveInsteadOfClass(&'s str),
}

impl<'s> fmt::Display for DemangleError<'s> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        todo!()
    }
}

impl error::Error for DemangleError<'_> {}
