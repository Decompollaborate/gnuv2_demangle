/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::{error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DemangleError<'s> {
    NonAscii,
    NotMangled,
    TrailingDataOnDestructor(&'s str),
    InvalidCustomNameCount(&'s str),
    RanOutOfCharactersForCustomName(&'s str),
    UnknownType(char),
    InvalidRepeatingArgument(&'s str),
    RanOutWhileDemanglingSpecial,
    RanOutOfArguments,
    InvalidSpecialMethod(&'s str),
    UnrecognizedSpecialMethod(&'s str),
    PrimitiveInsteadOfClass(&'s str),
    InvalidNamespaceCount(&'s str),
    InvalidLookbackCount(&'s str),
    LookbackCountTooBig(&'s str, usize),
    InvalidTypeOnTypeInfoFunction(&'s str),
    TrailingDataOnTypeInfoFunction(&'s str),
    InvalidTypeOnTypeInfoNode(&'s str),
    TrailingDataOnTypeInfoNode(&'s str),
}

impl<'s> fmt::Display for DemangleError<'s> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        todo!()
    }
}

impl error::Error for DemangleError<'_> {}
