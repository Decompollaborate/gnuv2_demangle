/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroUsize;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
};

use crate::{str_cutter::StrCutter, DemangleConfig, DemangleError};

use crate::{
    dem::demangle_custom_name,
    dem_arg::{demangle_argument, DemangledArg},
    dem_arg_list::ArgVec,
    dem_namespace::demangle_namespaces,
    remainer::{Remaining, StrParsing},
};

pub(crate) fn demangle_template<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &ArgVec,
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let Remaining { r, d: class_name } =
        demangle_custom_name(s, DemangleError::InvalidCustomNameOnTemplate)?;
    let Some(Remaining {
        r: remaining,
        d: digit,
    }) = r.p_digit()
    else {
        return Err(DemangleError::InvalidTemplateCount(r));
    };
    let digit = NonZeroUsize::new(digit).ok_or(DemangleError::TemplateReturnCountIsZero(r))?;

    let (remaining, types) = demangle_template_types_impl(config, remaining, digit, template_args)?;

    let templated = types.join();
    let template = if templated.ends_with('>') {
        format!("{}<{} >", class_name, templated)
    } else {
        format!("{}<{}>", class_name, templated)
    };
    Ok((remaining, template, class_name))
}

pub(crate) fn demangle_template_with_return_type<'c, 's>(
    config: &'c DemangleConfig,
    s: &'s str,
) -> Result<(&'s str, ArgVec<'c, 's>, Option<Cow<'s, str>>), DemangleError<'s>> {
    let Some(Remaining { r, d: digit }) = s.p_digit() else {
        return Err(DemangleError::InvalidTemplateReturnCount(s));
    };
    let digit = NonZeroUsize::new(digit).ok_or(DemangleError::TemplateReturnCountIsZero(s))?;

    let (r, types) = demangle_template_types_impl(config, r, digit, &ArgVec::new(config, None))?;

    let Some(r) = r.strip_prefix('_') else {
        return Err(DemangleError::MalformedTemplateWithReturnType(r));
    };
    let (r, namespaces) = if let Some(q_less) = r.strip_prefix('Q') {
        let (r, namespaces, _trailing_namespace) =
            demangle_namespaces(config, q_less, &ArgVec::new(config, None))?;

        (r, Some(Cow::from(namespaces)))
    } else if r.starts_with(|c| matches!(c, '1'..='9')) {
        let Remaining { r, d: namespace } =
            demangle_custom_name(r, DemangleError::InvalidNamespaceOnTemplatedFunction)?.d_as_cow();
        (r, Some(namespace))
    } else {
        (r, None)
    };

    Ok((r, types, namespaces))
}

fn demangle_template_types_impl<'c, 's>(
    config: &'c DemangleConfig,
    s: &'s str,
    count: NonZeroUsize,
    template_args: &ArgVec,
) -> Result<(&'s str, ArgVec<'c, 's>), DemangleError<'s>> {
    let mut remaining = s;

    let mut types = ArgVec::new(config, None);

    for _ in 0..count.get() {
        let (r, arg, allow_data_after_ellipsis) = if let Some(r) = remaining.strip_prefix('Z') {
            // typename / class
            let (r, arg) = demangle_argument(config, r, &types, template_args)?;
            (r, arg, true)
        } else {
            // value
            let Remaining { r, d: arg } =
                demangle_templated_value(config, remaining, template_args)?;
            (r, arg, false)
        };
        types.push(arg, remaining, r, allow_data_after_ellipsis)?;
        remaining = r;
    }

    Ok((remaining, types))
}

fn demangle_templated_value<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &ArgVec,
) -> Result<Remaining<'s, DemangledArg>, DemangleError<'s>> {
    let mut r = s;
    let mut is_pointer = false;
    let mut is_reference = false;

    // Skip over any known qualifier
    while !r.is_empty() {
        let c = r.chars().next().ok_or(DemangleError::RanOutOfArguments)?;

        match c {
            // '*'
            'P' => is_pointer = true,
            // '&'
            'R' => is_reference = true,
            // "const"
            'C' => {}
            // "signed" | "unsigned"
            'S' | 'U' => {}
            _ => break,
        }

        r = &r[1..];
    }

    let (remaining, arg) = if is_pointer || is_reference {
        let (aux, DemangledArg::Plain(_arg, _array_qualifiers)) = demangle_argument(
            config,
            r,
            &ArgVec::new(config, None),
            &ArgVec::new(config, None),
        )?
        else {
            return Err(DemangleError::InvalidTemplatedPointerReferenceValue(r));
        };
        let Remaining { r: aux, d: symbol } =
            demangle_custom_name(aux, DemangleError::InvalidSymbolNameOnTemplateType)?;
        let t = format!("{}{}", if is_pointer { "&" } else { "" }, symbol);
        (aux, DemangledArg::Plain(t, None.into()))
    } else {
        let remaining = r;
        let Remaining { r, d: c } = remaining
            .p_first()
            .ok_or(DemangleError::RanOutOfArguments)?;

        // Add a way to make clear which type is being used.
        match c {
            // "char" | "wchar_t"
            'c' | 'w' => {
                let Remaining { r, d: number } = r
                    .p_number()
                    .ok_or(DemangleError::InvalidTemplatedNumberForCharacterValue(r))?;
                let demangled_char = char::from_u32(
                    number
                        .try_into()
                        .map_err(|_| DemangleError::InvalidTemplatedCharacterValue(r, number))?,
                )
                .ok_or(DemangleError::InvalidTemplatedCharacterValue(r, number))?;
                let t = format!("'{demangled_char}'");
                (r, DemangledArg::Plain(t, None.into()))
            }
            // "short" | "int" | "long" | "long long"
            's' | 'i' | 'l' | 'x' => {
                if let Some(r) = r.strip_prefix('Y') {
                    // Y01 -> Use value at index 0 from the template list. No
                    // idea about the second digit

                    // TODO: what happens if the index is larger than 9?
                    let Some(Remaining { r, d: index }) = r.p_digit() else {
                        return Err(DemangleError::MissingLookbackIndexForTemplatedValue(s));
                    };
                    let Some(Remaining { r, d: digit1 }) = r.p_digit() else {
                        return Err(DemangleError::MissingLookbackSecondDigitForTemplatedValue(
                            s,
                        ));
                    };
                    if digit1 != 1 {
                        return Err(DemangleError::InvalidLookbackSecondDigitForTemplatedValue(
                            s, digit1,
                        ));
                    }

                    let Some(templated_value) = template_args.get(index) else {
                        return Err(DemangleError::IndexTooBigForYArgument(s, index));
                    };
                    (
                        r,
                        DemangledArg::Plain(templated_value.to_string(), None.into()),
                    )
                } else {
                    let (r, negative) = r.c_maybe_strip_prefix('m');
                    let Remaining { r, d: number } = if let Some(r) = r.strip_prefix('_') {
r
                        .p_number_maybe_multi_digit()
                        .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?
                    } else {
r
                        .p_number()
                        .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?
                    };
                    let t = format!("{}{}", if negative { "-" } else { "" }, number);
                    (r, DemangledArg::Plain(t, None.into()))
                }
            }
            // 'f' => {}, // "float"
            // 'd' => {}, // "double"
            // 'r' => {}, // "long double"
            // "bool"
            'b' => match r.chars().next() {
                Some('1') => (
                    &r[1..],
                    DemangledArg::Plain("true".to_string(), None.into()),
                ),
                Some('0') => (
                    &r[1..],
                    DemangledArg::Plain("false".to_string(), None.into()),
                ),
                _ => return Err(DemangleError::InvalidTemplatedBoolean(r)),
            },
            '1'..='9' => {
                // enum
                let Remaining { r, d: _enum_name } = demangle_custom_name(
                    remaining,
                    DemangleError::InvalidEnumNameForTemplatedValue,
                )?;

                // TODO: <(SomeEnum)0> is valid c++, try to use it somehow.

                let (r, negative) = r.c_maybe_strip_prefix('m');
                let Remaining { r, d: number } = r
                    .p_number()
                    .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?;
                let t = format!("{}{}", if negative { "-" } else { "" }, number);
                (r, DemangledArg::Plain(t, None.into()))
            }
            _ => return Err(DemangleError::InvalidTypeValueForTemplated(c, r)),
        }
    };

    Ok(Remaining::new(remaining, arg))
}
