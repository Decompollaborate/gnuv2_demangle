/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::{error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DemangleError<'s> {
    NotMangled,
    NonAscii,
    TrailingDataOnDestructor(&'s str),
    InvalidClassNameOnDestructor(&'s str),
    InvalidClassNameOnConstructor(&'s str),
    InvalidClassNameOnOperator(&'s str),
    InvalidClassNameOnMethod(&'s str),
    InvalidClassNameOnVirtualTable(&'s str),
    InvalidNamespaceOnNamespacedGlobal(&'s str),
    InvalidCustomNameOnArgument(&'s str),
    InvalidCustomNameOnNamespace(&'s str),
    InvalidCustomNameOnTemplate(&'s str),
    InvalidNamespaceOnTemplatedFunction(&'s str),
    InvalidSymbolNameOnTemplateType(&'s str),
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
    InvalidTemplateCount(&'s str),
    InvalidTemplateReturnCount(&'s str),
    MalformedTemplateWithReturnType(&'s str),
    // TODO: figure out what is X for and rename this
    InvalidValueForIndexOnXArgument(&'s str),
    InvalidValueForNumber1OnXArgument(&'s str),
    InvalidNumber1OnXArgument(&'s str, usize),
    IndexTooBigForXArgument(&'s str, usize),
    TrailingDataAfterArgumentList(&'s str),
    MalformedTemplateWithReturnTypeMissingReturnType(&'s str),
    MalformedTemplateWithReturnTypeMissingMalformedReturnType(&'s str),
    TrailingDataAfterReturnTypeOfMalformedTemplateWithReturnType(&'s str),
}

impl fmt::Display for DemangleError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        // TODO
        write!(
            f,
            "Sorry, I haven't implemented Display for DemangleError yet :c"
        )
    }
}

impl error::Error for DemangleError<'_> {}
