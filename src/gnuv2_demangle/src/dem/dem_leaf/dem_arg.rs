/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroU32;

use super::{
    demangle_arg_qualifiers, demangle_argument_list_impl, demangle_custom_name,
    demangle_namespaces, demangle_template,
};

use crate::{
    remainer::{Remaining, StrParsing},
    str_cutter::StrCutter,
    tree::leaf::{
        ArgQualifiers, ArgVec, DemangledArg, FunctionPointer, MemberPointer, MethodPointer,
        PlainArg, PlainArgArray, Signedness, TemplatedValueVec, Type, TypeMeta,
    },
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_argument<'s>(
    config: &DemangleConfig,
    full_args: &'s str,
    parsed_arguments: &ArgVec<'s>,
    template_args: &TemplatedValueVec<'s>,
    allow_array_fixup: bool,
) -> Result<(&'s str, DemangledArg<'s>), DemangleError<'s>> {
    if let Some(demangled) = demangle_qualifierless_arg(config, full_args)? {
        return Ok(demangled);
    }

    let Remaining {
        r: args,
        d: qualifiers,
    } = demangle_arg_qualifiers(config, full_args, allow_array_fixup)?;

    if let Some(s) = args.strip_prefix('F') {
        let (r, fp) =
            demangle_function_pointer_arg(config, s, template_args, qualifiers, allow_array_fixup)?;
        Ok((r, DemangledArg::FunctionPointer(fp)))
    } else if let Some(r) = args.strip_prefix('M') {
        let (r, mp) = demangle_method_pointer_arg(
            config,
            r,
            full_args,
            template_args,
            qualifiers,
            allow_array_fixup,
        )?;
        Ok((r, DemangledArg::MethodPointer(mp)))
    } else if let Some(r) = args.strip_prefix('O') {
        let (r, mp) = demangle_object_pointer_arg(
            config,
            r,
            full_args,
            template_args,
            qualifiers,
            allow_array_fixup,
        )?;
        Ok((r, DemangledArg::MemberPointer(mp)))
    } else {
        // 'G' is used for classes, structs and unions, so we must make sure we
        // don't parse a primitive type next, otherwise this is not properly
        // mangled.
        let (args, must_be_class_like) = args.c_maybe_strip_prefix('G');

        let ArgQualifiers {
            sign,
            post_qualifiers,
            pre_qualifiers,
            array_qualifiers,
        } = qualifiers;

        let Remaining { r, d: typ } = demangle_arg_type(
            config,
            args,
            sign,
            parsed_arguments,
            template_args,
            allow_array_fixup,
        )?;

        if must_be_class_like && !typ.class_like() {
            return Err(DemangleError::PrimitiveInsteadOfClass(full_args));
        }

        let pa = PlainArg::new(typ, post_qualifiers, pre_qualifiers);
        let paa = PlainArgArray::new(pa, array_qualifiers);

        Ok((r, DemangledArg::Plain(paa)))
    }
}

fn demangle_arg_type<'s>(
    config: &DemangleConfig,
    args: &'s str,
    mut sign: Signedness,
    parsed_arguments: &ArgVec<'s>,
    template_args: &TemplatedValueVec<'s>,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, TypeMeta<'s>>, DemangleError<'s>> {
    let c = args
        .chars()
        .next()
        .ok_or(DemangleError::RanOutOfArguments)?;

    let (args, typ) = match c {
        'c' => (&args[1..], Type::Char),
        's' => (&args[1..], Type::Short),
        'i' => (&args[1..], Type::Int),
        'l' => (&args[1..], Type::Long),
        'x' => (&args[1..], Type::LongLong),
        'f' => (&args[1..], Type::Float),
        'd' => (&args[1..], Type::Double),
        'r' => (&args[1..], Type::LongDouble),
        'b' => (&args[1..], Type::Bool),
        'w' => (&args[1..], Type::WChar),
        'v' => (&args[1..], Type::Void),
        'I' => {
            let Remaining { r, d: bitwidth } = args[1..].p_hex_number().ok_or(
                DemangleError::MissingBitwidthForExtensionInteger(&args[1..]),
            )?;
            let typ = match bitwidth {
                128 => {
                    // g++ does not like the `int128_t` type, but it recognizes
                    // `__int128_t` and `__uint128_t` just fine, so we emit
                    // instead.
                    // Also `unsigned __int128_t` doesn't make sense. Some g++
                    // versions kinda recognizes it, but it mangles the symbol
                    // as `unsigned int`, so it seems more like a bug than an
                    // actual feature.
                    if config.fix_extension_int {
                        if sign == Signedness::Unsigned {
                            sign = Signedness::No;
                            Type::UInt128
                        } else {
                            Type::SInt128
                        }
                    } else {
                        Type::Int128
                    }
                }
                _ => {
                    return Err(DemangleError::InvalidBitwidthForExtensionInteger(
                        args, bitwidth,
                    ));
                }
            };
            (r, typ)
        }
        '1'..='9' => {
            let Remaining { r, d: class_name } =
                demangle_custom_name(args, DemangleError::InvalidCustomNameOnArgument)?;
            (r, Type::Class(class_name))
        }
        'Q' => {
            let Remaining {
                r: remaining,
                d: ns,
            } = demangle_namespaces(config, &args[1..], template_args, allow_array_fixup)?;
            (remaining, Type::Namespaces(ns))
        }
        'T' => {
            // Remembered type / look back
            let Remaining { r, d: lookback } = args[1..]
                .p_number_maybe_multi_digit()
                .ok_or(DemangleError::InvalidLookbackCount(args))?;

            let referenced_arg = parsed_arguments.get(lookback as usize).ok_or(
                DemangleError::LookbackCountTooBig(args, lookback, parsed_arguments.len()),
            )?;

            (
                r,
                Type::LookBack {
                    referenced_arg,
                    lookback,
                },
            )
        }
        't' => {
            // templates
            let Remaining {
                r: remaining,
                d: template,
            } = demangle_template(config, &args[1..], template_args, allow_array_fixup)?;
            (remaining, Type::Template(template))
        }
        'X' => {
            // Index into type of templated function
            let args = &args[1..];
            let Remaining { r, d: index } = if let Some(r) = args.strip_prefix('_') {
                r.p_number_maybe_multi_digit()
            } else {
                args.p_digit()
            }
            .ok_or(DemangleError::InvalidValueForIndexOnXArgument(args))?;

            let Some(Remaining { r, d: number1 }) = r.p_digit() else {
                return Err(DemangleError::InvalidValueForNumber1OnXArgument(r));
            };
            // TODO: what is this number?
            if number1 != 1 && number1 != 0 {
                return Err(DemangleError::InvalidNumber1OnXArgument(r, number1));
            }

            let Some(referenced_arg) = template_args.get(index as usize) else {
                return Err(DemangleError::IndexTooBigForXArgument(r, index));
            };

            (
                r,
                Type::TemplatedFunctionIndex {
                    referenced_arg,
                    lookback: index,
                },
            )
        }
        _ => {
            return Err(DemangleError::UnknownType(c, args));
        }
    };

    let t = TypeMeta::new(sign, typ);

    Ok(Remaining::new(args, t))
}

/// Handles any arg that can't be qualified
fn demangle_qualifierless_arg<'s>(
    _config: &DemangleConfig,
    full_args: &'s str,
) -> Result<Option<(&'s str, DemangledArg<'s>)>, DemangleError<'s>> {
    #[expect(clippy::manual_map)]
    let maybe_demangled = if let Some(repeater) = full_args.strip_prefix('N') {
        let remaining = repeater;
        let Remaining {
            r: remaining,
            d: count,
        } = remaining
            .p_number_maybe_multi_digit()
            .ok_or(DemangleError::InvalidRepeatingArgument(full_args))?;
        let count =
            NonZeroU32::new(count).ok_or(DemangleError::InvalidRepeatingArgument(full_args))?;

        let Remaining {
            r: remaining,
            d: index,
        } = remaining
            .p_number_maybe_multi_digit()
            .ok_or(DemangleError::InvalidRepeatingArgument(full_args))?;

        Some((remaining, DemangledArg::Repeat { count, index }))
    } else if let Some(remaining) = full_args.strip_prefix('e') {
        Some((remaining, DemangledArg::Ellipsis))
    } else {
        None
    };

    Ok(maybe_demangled)
}

/// Function pointer/reference
fn demangle_function_pointer_arg<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &TemplatedValueVec<'s>,
    qualifiers: ArgQualifiers,
    allow_array_fixup: bool,
) -> Result<(&'s str, FunctionPointer<'s>), DemangleError<'s>> {
    let Remaining { r, d: func_args } =
        demangle_argument_list_impl(config, s, None, template_args, true, allow_array_fixup)?;
    let Some(r) = r.strip_prefix('_') else {
        return Err(DemangleError::MissingReturnTypeForFunctionPointer(r));
    };

    let (r, return_type) =
        demangle_argument(config, r, &func_args, template_args, allow_array_fixup)?;

    let ArgQualifiers {
        sign,
        post_qualifiers,
        pre_qualifiers,
        array_qualifiers,
    } = qualifiers;

    if sign != Signedness::No {
        return Err(DemangleError::InvalidSignednessForFunctionPointer(s));
    }
    if array_qualifiers.is_some() {
        return Err(DemangleError::InvalidArrayQualifiersForFunctionPointer(s));
    }

    let (ret, array_qualifiers) = match return_type {
        DemangledArg::Plain(plain_array) => {
            let PlainArgArray {
                plain,
                array_qualifiers,
            } = plain_array;

            (plain.into(), array_qualifiers)
        }
        DemangledArg::FunctionPointer(function_pointer) => (function_pointer.into(), None),
        DemangledArg::MethodPointer(method_pointer) => (method_pointer.into(), None),
        DemangledArg::MemberPointer(member_pointer) => (member_pointer.into(), None),
        DemangledArg::Repeat { .. } | DemangledArg::Ellipsis => {
            return Err(DemangleError::InvalidReturnTypeForFunctionPointer(r))
        }
    };

    let fp = FunctionPointer::new(
        ret,
        array_qualifiers,
        post_qualifiers,
        pre_qualifiers,
        func_args,
    );
    Ok((r, fp))
}

/// Method pointer
fn demangle_method_pointer_arg<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_args: &'s str,
    template_args: &TemplatedValueVec<'s>,
    qualifiers: ArgQualifiers,
    allow_array_fixup: bool,
) -> Result<(&'s str, MethodPointer<'s>), DemangleError<'s>> {
    if qualifiers.sign != Signedness::No
        || !qualifiers.post_qualifiers.valid_as_method_pointer_arg()
    {
        // The only qualifer valid for this seems to be pointer (`*`), not
        // even references (`&`) seem to be valid C++
        return Err(DemangleError::InvalidQualifierForMethodMemberArg(full_args));
    }

    let (
        r,
        DemangledArg::Plain(PlainArgArray {
            plain: class_name,
            array_qualifiers: None,
        }),
    ) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        template_args,
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidClassNameOnMethodArgument(s));
    };

    let (r, is_const_method) = r.c_maybe_strip_prefix('C');
    if let Some(func_pointer) = r.strip_prefix('F') {
        let r = {
            // First argument should be a pointer to the class name.
            // We can't do much about it, besides use it check the mangled name
            // is valid.

            // It has to be a pointer
            let Some(r) = func_pointer.strip_prefix('P') else {
                return Err(DemangleError::MethodPointerNotHavingAPointerFirst(
                    func_pointer,
                ));
            };

            // Possibly `const`, but only if the class is const.
            let r = if is_const_method {
                r.strip_prefix('C')
                    .ok_or(DemangleError::MethodPointerMissingConstness(func_pointer))?
            } else {
                r
            };

            let (r, DemangledArg::Plain(paa)) = demangle_argument(
                config,
                r,
                &ArgVec::new(config, None),
                template_args,
                allow_array_fixup,
            )?
            else {
                return Err(DemangleError::MissingFirstClassArgumentForMethodMemberArg(
                    func_pointer,
                ));
            };

            let PlainArgArray {
                plain: class_name_again,
                array_qualifiers,
            } = paa;

            if class_name != class_name_again {
                return Err(DemangleError::MethodPointerWrongClassName(func_pointer));
            }
            if array_qualifiers.is_some() {
                return Err(DemangleError::MethodPointerClassNameAsArray(func_pointer));
            }
            r
        };

        let (r, fp) =
            demangle_function_pointer_arg(config, r, template_args, qualifiers, allow_array_fixup)?;

        let arg = MethodPointer::new(fp, class_name, is_const_method);
        Ok((r, arg))
    } else {
        // What else could this be?
        Err(DemangleError::UnknownMethodMemberArgKind(r))
    }
}

/// Object pointer
fn demangle_object_pointer_arg<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_args: &'s str,
    template_args: &TemplatedValueVec<'s>,
    qualifiers: ArgQualifiers,
    allow_array_fixup: bool,
) -> Result<(&'s str, MemberPointer<'s>), DemangleError<'s>> {
    let ArgQualifiers {
        sign: Signedness::No,
        post_qualifiers,
        pre_qualifiers,
        array_qualifiers: None,
    } = qualifiers
    else {
        // The only qualifer valid for this seems to be pointer (`*`), not
        // even references (`&`) seem to be valid C++
        return Err(DemangleError::InvalidQualifierForObjectMemberArg(full_args));
    };

    if !post_qualifiers.valid_as_method_pointer_arg() {
        // The only qualifer valid for this seems to be pointer (`*`), not
        // even references (`&`) seem to be valid C++
        return Err(DemangleError::InvalidQualifierForObjectMemberArg(full_args));
    }

    let (
        r,
        DemangledArg::Plain(PlainArgArray {
            plain: class_name,
            array_qualifiers: None,
        }),
    ) = demangle_argument(
        config,
        s,
        &ArgVec::new(config, None),
        template_args,
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidClassNameOnObjectMemberArgument(s));
    };

    let Some(r) = r.strip_prefix('_') else {
        return Err(DemangleError::MissingTypeForObjectMemberPointer(r));
    };

    let (r, DemangledArg::Plain(member_type)) = demangle_argument(
        config,
        r,
        &ArgVec::new(config, None),
        template_args,
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidTypeForObjectMemberPointer(full_args));
    };

    let PlainArgArray {
        plain: member_type,
        array_qualifiers,
    } = member_type;

    let op = MemberPointer::new(
        class_name,
        member_type,
        array_qualifiers,
        post_qualifiers,
        pre_qualifiers,
    );

    Ok((r, op))
}
