/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::{error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DemangleError<'s> {
    NonAscii,
    NotMangled,
    TrailingDataOnDestructor(&'s str),
    InvalidCustomNameCount(&'s str),
    RanOutOfCharactersForCustomName(&'s str),
    UnknownType(char, &'s str),
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
    TrailingDataAfterEllipsis(&'s str),
    InvalidTypeValueForTemplated(char, &'s str),
    InvalidValueForIntegralTemplated(&'s str),
    InvalidTemplatedPointerReferenceValue(&'s str),
    InvalidTemplatedNumberForCharacterValue(&'s str),
    InvalidTemplatedCharacterValue(&'s str, usize),
    InvalidTemplatedBoolean(&'s str),
    VTableMissingDollarSeparator(&'s str),
    InvalidNamespacedGlobal(&'s str, &'s str),
    TrailingDataOnNamespacedGlobal(&'s str),
    MissingReturnTypeForFunctionPointer(&'s str),
    InvalidReturnTypeForFunctionPointer(&'s str),
    InvalidGlobalSymKeyed(&'s str),
    InvalidArraySize(&'s str),
    MalformedArrayArgumment(&'s str),
    MalformedCastOperatorOverload(&'s str),
}

impl<'s> fmt::Display for DemangleError<'s> {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        todo!()
    }
}

impl error::Error for DemangleError<'_> {}
