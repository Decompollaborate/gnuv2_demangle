/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    borrow::Cow,
    string::{String, ToString},
};
use core::num::NonZeroUsize;

use super::{
    demangle_arg_qualifiers, demangle_argument_list_impl, demangle_custom_name,
    demangle_namespaces, demangle_template,
};

use crate::{
    remainer::{Remaining, StrParsing},
    str_cutter::StrCutter,
    tree::leaf::{
        ArgQualifiers, ArgVec, DemangledArg, FunctionPointer, MethodPointer, Signedness, Type,
        TypeMeta,
    },
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_argument<'s>(
    config: &DemangleConfig,
    full_args: &'s str,
    parsed_arguments: &ArgVec<'s>,
    template_args: &ArgVec<'s>,
    allow_array_fixup: bool,
) -> Result<(&'s str, DemangledArg), DemangleError<'s>> {
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
        Ok((r, DemangledArg::ObjectPointer(mp)))
    } else {
        // 'G' is used for classes, structs and unions, so we must make sure we
        // don't parse a primitive type next, otherwise this is not properly
        // mangled.
        let (args, must_be_class_like) = args.c_maybe_strip_prefix('G');

        let ArgQualifiers {
            sign,
            post_qualifiers,
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
        let sign = &typ.sign();

        let out = format!(
            "{}{}{}{}",
            sign,
            typ.typ(),
            if !post_qualifiers.is_empty() { " " } else { "" },
            post_qualifiers.to_string().trim_matches(' ')
        );

        Ok((r, DemangledArg::Plain(out, array_qualifiers)))
    }
}

fn demangle_arg_type<'s>(
    config: &DemangleConfig,
    args: &'s str,
    mut sign: Signedness,
    parsed_arguments: &ArgVec<'s>,
    template_args: &ArgVec<'s>,
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

            let referenced_arg =
                parsed_arguments
                    .get(lookback)
                    .ok_or(DemangleError::LookbackCountTooBig(
                        args,
                        lookback,
                        parsed_arguments.len(),
                    ))?;

            let referenced_arg = Cow::from(referenced_arg);
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

            let Some(t) = template_args.get(index) else {
                return Err(DemangleError::IndexTooBigForXArgument(r, index));
            };

            let referenced_arg = Cow::from(t);
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
) -> Result<Option<(&'s str, DemangledArg)>, DemangleError<'s>> {
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
            NonZeroUsize::new(count).ok_or(DemangleError::InvalidRepeatingArgument(full_args))?;

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
    template_args: &ArgVec<'s>,
    qualifiers: ArgQualifiers,
    allow_array_fixup: bool,
) -> Result<(&'s str, FunctionPointer), DemangleError<'s>> {
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
        array_qualifiers,
    } = qualifiers;

    let fp = match return_type {
        DemangledArg::Plain(plain, array_qualifiers) => FunctionPointer {
            return_type: format!("{sign}{plain}"),
            array_qualifiers,
            post_qualifiers: post_qualifiers.to_string(),
            args: func_args.join(),
        },
        DemangledArg::FunctionPointer(function_pointer) => {
            let FunctionPointer {
                return_type: sub_return_type,
                array_qualifiers: sub_array_qualifiers,
                post_qualifiers: sub_post_qualifiers,
                args: sub_args,
            } = function_pointer;
            let func_args = func_args.join();
            FunctionPointer {
                return_type: sub_return_type,
                array_qualifiers: sub_array_qualifiers,
                // This is kinda hacky, but it seems to work...
                post_qualifiers: format!(
                    "{sign}{post_qualifiers}({sub_post_qualifiers})({func_args}){array_qualifiers}",
                ),
                args: sub_args,
            }
        }
        DemangledArg::MethodPointer(method_pointer) => {
            // Copied from the FunctionPointer block. Untested
            let MethodPointer {
                return_type: sub_return_type,
                array_qualifiers: sub_array_qualifiers,
                class,
                post_qualifiers: sub_post_qualifiers,
                args: sub_args,
                is_const_method,
            } = method_pointer;
            let func_args = func_args.join();
            let const_qualifier = if is_const_method { " const" } else { "" };
            FunctionPointer {
                return_type: sub_return_type,
                array_qualifiers: sub_array_qualifiers,
                post_qualifiers: format!(
                    "{sign}{post_qualifiers}({class}::{sub_post_qualifiers})({func_args}){const_qualifier}{array_qualifiers}",
                ),
                args: sub_args,
            }
        }
        DemangledArg::ObjectPointer(object) => FunctionPointer {
            return_type: format!("{sign}{object}"),
            array_qualifiers: None.into(),
            post_qualifiers: post_qualifiers.to_string(),
            args: func_args.join(),
        },
        DemangledArg::Repeat { .. } | DemangledArg::Ellipsis => {
            return Err(DemangleError::InvalidReturnTypeForFunctionPointer(r))
        }
    };

    Ok((r, fp))
}

/// Method pointer
fn demangle_method_pointer_arg<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_args: &'s str,
    template_args: &ArgVec<'s>,
    qualifiers: ArgQualifiers,
    allow_array_fixup: bool,
) -> Result<(&'s str, MethodPointer), DemangleError<'s>> {
    if qualifiers.sign != Signedness::No
        || !qualifiers.post_qualifiers.valid_as_method_pointer_arg()
    {
        // The only qualifer valid for this seems to be pointer (`*`), not
        // even references (`&`) seem to be valid C++
        return Err(DemangleError::InvalidQualifierForMethodMemberArg(full_args));
    }

    let (r, class_name) = if s.starts_with(|c| matches!(c, '1'..='9')) {
        let Remaining { r, d: class_name } =
            demangle_custom_name(s, DemangleError::InvalidClassNameOnMethodArgument)?;
        (r, Cow::from(class_name))
    } else {
        let (r, DemangledArg::Plain(class_name, array_qualifiers)) = demangle_argument(
            config,
            s,
            &ArgVec::new(config, None),
            template_args,
            allow_array_fixup,
        )?
        else {
            return Err(DemangleError::InvalidClassNameOnMethodArgument(s));
        };
        if array_qualifiers.is_some() {
            return Err(DemangleError::InvalidClassNameOnMethodArgument(s));
        }

        (r, Cow::from(class_name))
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

            let (r, DemangledArg::Plain(class_name_again, array_qualifiers)) = demangle_argument(
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
        let FunctionPointer {
            return_type,
            array_qualifiers,
            post_qualifiers,
            args,
        } = fp;

        let arg = MethodPointer {
            return_type,
            array_qualifiers,
            class: class_name.to_string(),
            post_qualifiers,
            args,
            is_const_method,
        };
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
    template_args: &ArgVec<'s>,
    qualifiers: ArgQualifiers,
    allow_array_fixup: bool,
) -> Result<(&'s str, String), DemangleError<'s>> {
    if qualifiers.sign != Signedness::No
        || !qualifiers.post_qualifiers.valid_as_method_pointer_arg()
        || qualifiers.array_qualifiers.is_some()
    {
        // The only qualifer valid for this seems to be pointer (`*`), not
        // even references (`&`) seem to be valid C++
        return Err(DemangleError::InvalidQualifierForObjectMemberArg(full_args));
    }

    let (r, class_name) = if s.starts_with(|c| matches!(c, '1'..='9')) {
        let Remaining { r, d: class_name } =
            demangle_custom_name(s, DemangleError::InvalidClassNameOnObjectMemberArgument)?;
        (r, Cow::from(class_name))
    } else {
        let (r, DemangledArg::Plain(class_name, array_qualifiers)) = demangle_argument(
            config,
            s,
            &ArgVec::new(config, None),
            template_args,
            allow_array_fixup,
        )?
        else {
            return Err(DemangleError::InvalidClassNameOnObjectMemberArgument(s));
        };
        if array_qualifiers.is_some() {
            return Err(DemangleError::InvalidClassNameOnObjectMemberArgument(s));
        }

        (r, Cow::from(class_name))
    };

    let Some(r) = r.strip_prefix('_') else {
        return Err(DemangleError::MissingTypeForObjectMemberPointer(r));
    };

    let (r, DemangledArg::Plain(member_type, arr)) = demangle_argument(
        config,
        r,
        &ArgVec::new(config, None),
        template_args,
        allow_array_fixup,
    )?
    else {
        return Err(DemangleError::InvalidTypeForObjectMemberPointer(full_args));
    };

    // Arrays makes everything harder.
    let mut arg = member_type;
    arg.push(' ');
    if let Some(arr) = arr.as_option() {
        if !arr.inner_post_qualifiers.is_empty() {
            arg.push('(');
            arg.push_str(&arr.inner_post_qualifiers.to_string());
        }
    }
    let post_qualifiers = qualifiers.post_qualifiers;
    arg += &format!("({class_name}::{post_qualifiers})");
    if let Some(arr) = arr.as_option() {
        if !arr.inner_post_qualifiers.is_empty() {
            arg.push(')');
        }
        arg.push_str(&arr.arrays);
    }

    Ok((r, arg))
}
