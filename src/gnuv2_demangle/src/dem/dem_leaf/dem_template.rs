/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroU32;

use super::{demangle_argument, demangle_custom_name, demangle_namespaces};

use crate::{
    remainer::{Remaining, StrParsing},
    str_cutter::StrCutter,
    tree::leaf::{
        ArgVec, DemangledArg, FunctionPointer, NamespaceKind, NamespacedType, Template,
        TemplateElement, TemplatedTypename, TemplatedValue, TemplatedValueVec,
    },
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_template<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &TemplatedValueVec<'s>,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, Template<'s>>, DemangleError<'s>> {
    let Remaining { r, d: class_name } =
        demangle_custom_name(s, DemangleError::InvalidCustomNameOnTemplate)?;
    let Some(Remaining {
        r: remaining,
        d: digit,
    }) = r.p_digit()
    else {
        return Err(DemangleError::InvalidTemplateCount(r));
    };
    let digit = NonZeroU32::new(digit).ok_or(DemangleError::TemplateReturnCountIsZero(r))?;

    let (remaining, types) =
        demangle_template_types_impl(config, remaining, digit, template_args, allow_array_fixup)?;

    let templated = types.join();

    let t = Template::new(templated, class_name);

    Ok(Remaining::new(remaining, t))
}

pub(crate) fn demangle_template_with_return_type<'s>(
    config: &DemangleConfig,
    s: &'s str,
    allow_array_fixup: bool,
) -> Result<(&'s str, TemplatedValueVec<'s>, Option<NamespacedType<'s>>), DemangleError<'s>> {
    let Some(Remaining { r, d: digit }) = s.p_digit() else {
        return Err(DemangleError::InvalidTemplateReturnCount(s));
    };
    let digit = NonZeroU32::new(digit).ok_or(DemangleError::TemplateReturnCountIsZero(s))?;

    let (r, types) = demangle_template_types_impl(
        config,
        r,
        digit,
        &TemplatedValueVec::new(),
        allow_array_fixup,
    )?;

    let Some(r) = r.strip_prefix('_') else {
        return Err(DemangleError::MalformedTemplateWithReturnType(r));
    };
    let (r, namespaces) = if let Some(q_less) = r.strip_prefix('Q') {
        let Remaining { r, d: nt } =
            demangle_namespaces(config, q_less, &TemplatedValueVec::new(), allow_array_fixup)?;

        (r, Some(nt))
    } else if r.starts_with(|c| matches!(c, '1'..='9')) {
        let Remaining { r, d: classname } =
            demangle_custom_name(r, DemangleError::InvalidNamespaceOnTemplatedFunction)?;

        let nt = NamespacedType {
            namespace: NamespaceKind::Class(classname),
            type_name: classname,
        };

        (r, Some(nt))
    } else {
        (r, None)
    };

    Ok((r, types, namespaces))
}

fn demangle_template_types_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    count: NonZeroU32,
    template_args: &TemplatedValueVec<'s>,
    allow_array_fixup: bool,
) -> Result<(&'s str, TemplatedValueVec<'s>), DemangleError<'s>> {
    let mut remaining = s;
    let mut types = TemplatedValueVec::new();

    for _i in 0..count.get() {
        let (r, element) = if let Some(r) = remaining.strip_prefix('Z') {
            // typename / class
            let (r, arg) = demangle_argument(
                config,
                r,
                &ArgVec::new(config, None),
                template_args,
                allow_array_fixup,
            )?;

            let typename = match arg {
                DemangledArg::Plain(p) => TemplatedTypename::Plain(p),
                DemangledArg::FunctionPointer(function_pointer) => {
                    TemplatedTypename::FunctionPointer(function_pointer)
                }
                DemangledArg::MethodPointer(method_pointer) => {
                    TemplatedTypename::MethodPointer(method_pointer)
                }
                DemangledArg::MemberPointer(op) => TemplatedTypename::MemberPointer(op),
                DemangledArg::Repeat { count, index } => {
                    return Err(
                        DemangleError::UnexpectedTypeRepetitionOnTemplateTypenameList(
                            s, count, index,
                        ),
                    )
                }
                DemangledArg::Ellipsis => {
                    return Err(DemangleError::UnexpectedEllipsisOnOnTemplateTypenameList(s))
                }
            };
            (r, TemplateElement::Typename(typename))
        } else {
            // value
            let Remaining { r, d: value } =
                demangle_templated_value(config, remaining, template_args, allow_array_fixup)?;
            (r, TemplateElement::Value(value))
        };
        types.push(element);
        remaining = r;
    }

    Ok((remaining, types))
}

fn demangle_templated_value<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &TemplatedValueVec<'s>,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, TemplatedValue<'s>>, DemangleError<'s>> {
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
        let (aux, demangled_arg) = demangle_argument(
            config,
            r,
            &ArgVec::new(config, None),
            &TemplatedValueVec::new(),
            allow_array_fixup,
        )?;

        match demangled_arg {
            DemangledArg::Plain(..) => {
                let Remaining { r: aux, d: symbol } =
                    demangle_custom_name(aux, DemangleError::InvalidSymbolNameOnTemplateType)?;
                let ampersand = if is_pointer { "&" } else { "" };
                let t = format!("{ampersand}{symbol}");
                (aux, TemplatedValue::ObjectReference(t))
            }
            DemangledArg::FunctionPointer(function_pointer) => {
                // Function pointers as types in template lists

                let FunctionPointer {
                    return_type,
                    array_qualifiers: _,
                    post_qualifiers: _,
                    args,
                } = function_pointer;

                let Remaining { r: aux, d: symbol } =
                    demangle_custom_name(aux, DemangleError::InvalidSymbolNameOnTemplateType)?;
                let symbol = symbol.into_raw();

                // TODO: check `_mangled_args` demangles to `args`
                let Some((actual_sym, _mangled_args)) = symbol.c_split2("__F") else {
                    return Err(DemangleError::InvalidFunctionPointerTypeInTemplatedList(
                        r, symbol,
                    ));
                };

                let ampersand = if is_pointer { "&" } else { "" };
                let t = if config.fix_function_pointers_in_template_lists {
                    if is_pointer {
                        TemplatedValue::FunctionPointer(format!(
                            "({return_type}(*)({args})) {ampersand}{actual_sym}"
                        ))
                    } else {
                        TemplatedValue::FunctionReference(format!(
                            "({return_type}(&)({args})) {ampersand}{actual_sym}"
                        ))
                    }
                } else {
                    TemplatedValue::FunctionRaw(format!("{ampersand}{actual_sym}({args})"))
                };
                (aux, t)
            }
            DemangledArg::MemberPointer(..) => {
                // TODO: merge with DemangledArg::Plain ?
                // TODO: does it make sense to have them both?
                let Remaining { r: aux, d: symbol } =
                    demangle_custom_name(aux, DemangleError::InvalidSymbolNameOnTemplateType)?;
                let ampersand = if is_pointer { "&" } else { "" };
                let t = format!("{ampersand}{symbol}");
                (aux, TemplatedValue::ObjectReference(t))
            }
            DemangledArg::MethodPointer(..)
            | DemangledArg::Repeat { .. }
            | DemangledArg::Ellipsis => {
                return Err(DemangleError::InvalidTemplatedPointerReferenceValue(r))
            }
        }
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
                let demangled_char = char::from_u32(number)
                    .ok_or(DemangleError::InvalidTemplatedCharacterValue(r, number))?;
                (r, TemplatedValue::Character(demangled_char))
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

                    let Some(templated_value) = template_args.get(index as usize) else {
                        return Err(DemangleError::IndexTooBigForYArgument(s, index));
                    };
                    (r, TemplatedValue::LookbackInt(templated_value))
                } else {
                    let (r, negative) = r.c_maybe_strip_prefix('m');
                    let Remaining { r, d: number } = if let Some(r) = r.strip_prefix('_') {
                        r.p_number_maybe_multi_digit()
                            .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?
                    } else {
                        r.p_number()
                            .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?
                    };
                    (r, TemplatedValue::Int { number, negative })
                }
            }
            // 'f' => {}, // "float"
            // 'd' => {}, // "double"
            // 'r' => {}, // "long double"
            // "bool"
            'b' => match r.chars().next() {
                Some('1') => (&r[1..], TemplatedValue::Bool(true)),
                Some('0') => (&r[1..], TemplatedValue::Bool(false)),
                _ => return Err(DemangleError::InvalidTemplatedBoolean(r)),
            },
            '1'..='9' => {
                // enum
                let Remaining { r, d: enum_name } = demangle_custom_name(
                    remaining,
                    DemangleError::InvalidEnumNameForTemplatedValue,
                )?;

                let (r, negative) = r.c_maybe_strip_prefix('m');
                let Remaining { r, d: number } = r
                    .p_number()
                    .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?;

                let tv = TemplatedValue::Enum {
                    enum_name,
                    number,
                    negative,
                };
                (r, tv)
            }
            _ => return Err(DemangleError::InvalidTypeValueForTemplated(c, r)),
        }
    };

    Ok(Remaining::new(remaining, arg))
}
