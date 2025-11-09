/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::fmt;
use core::num::NonZeroUsize;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
};

use crate::{option_display::OptionDisplay, str_cutter::StrCutter};
use crate::{DemangleConfig, DemangleError};

use crate::{
    dem::demangle_custom_name,
    dem_arg_list::{demangle_argument_list_impl, ArgVec},
    dem_namespace::demangle_namespaces,
    dem_template::demangle_template,
    remainer::{Remaining, StrParsing},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DemangledArg {
    Plain(String, OptionDisplay<ArrayQualifiers>),
    FunctionPointer(FunctionPointer),
    MethodPointer(MethodPointer),
    Repeat { count: NonZeroUsize, index: usize },
    Ellipsis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FunctionPointer {
    pub(crate) return_type: String,
    pub(crate) array_qualifiers: OptionDisplay<ArrayQualifiers>,
    pub(crate) post_qualifiers: String,
    pub(crate) args: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MethodPointer {
    pub(crate) return_type: String,
    pub(crate) array_qualifiers: OptionDisplay<ArrayQualifiers>,
    pub(crate) class: String, // TODO: `&'s str` instead? should be easy, i think...
    pub(crate) post_qualifiers: String,
    pub(crate) args: String,
    pub(crate) is_const_method: bool,
}

impl fmt::Display for FunctionPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let FunctionPointer {
            return_type,
            array_qualifiers,
            post_qualifiers,
            args,
        } = self;

        let array_qualifiers = array_qualifiers.as_option();
        let mut wrote_space = false;

        write!(f, "{return_type}")?;
        if let Some(arr) = array_qualifiers {
            // A return type being a pointer to array has a terrible syntax,
            // which we need to break up.
            // Instead of writing `int (*)[3] (*)(void)` (function pointer
            // returning a pointer to array), we have to instead write
            // `int (*(*)(void))[3]`.

            write!(f, " ")?;
            wrote_space = true;
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, "({}", arr.inner_post_qualifiers)?;
            }
        }
        if !return_type.ends_with(['*', '&']) && !wrote_space {
            write!(f, " ")?;
        }
        write!(f, "({})", post_qualifiers.trim_matches(' '))?;
        write!(f, "({args})")?;
        if let Some(arr) = array_qualifiers {
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, ")")?;
            }
            write!(f, "{}", arr.arrays)?;
        }
        Ok(())
    }
}

impl fmt::Display for MethodPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MethodPointer {
            return_type,
            array_qualifiers,
            class,
            post_qualifiers,
            args,
            is_const_method,
        } = self;

        let array_qualifiers = array_qualifiers.as_option();
        let mut wrote_space = false;

        write!(f, "{return_type}")?;
        if let Some(arr) = array_qualifiers {
            // A return type being a pointer to array has a terrible syntax,
            // which we need to break up.
            // Instead of writing `int (*)[3] (*)(void)` (function pointer
            // returning a pointer to array), we have to instead write
            // `int (*(*)(void))[3]`.

            write!(f, " ")?;
            wrote_space = true;
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, "({}", arr.inner_post_qualifiers)?;
            }
        }
        if !return_type.ends_with(['*', '&']) && !wrote_space {
            write!(f, " ")?;
        }
        write!(f, "({}::{})", class, post_qualifiers.trim_matches(' '))?;
        write!(f, "({args})")?;
        if *is_const_method {
            write!(f, " const")?;
        }
        if let Some(arr) = array_qualifiers {
            if !arr.inner_post_qualifiers.is_empty() {
                write!(f, ")")?;
            }
            write!(f, "{}", arr.arrays)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Signedness {
    No,
    Signed,
    Unsigned,
}

impl fmt::Display for Signedness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => Ok(()),
            Self::Signed => write!(f, "signed "),
            Self::Unsigned => write!(f, "unsigned "),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArrayQualifiers {
    pub(crate) inner_post_qualifiers: String,
    // TODO: would be better to store this as a vector instead of an array?
    pub(crate) arrays: String,
}

impl fmt::Display for ArrayQualifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")?;

        if !self.inner_post_qualifiers.is_empty() {
            // Only add parenthesis if there are post_qualifiers, like a
            // pointer.
            // Arrays without being decaying to pointers can happen in, for
            // example, templated functions.
            write!(f, "({})", self.inner_post_qualifiers)?;
        }

        write!(f, "{}", self.arrays)
    }
}

pub(crate) fn demangle_argument<'s>(
    config: &DemangleConfig,
    full_args: &'s str,
    parsed_arguments: &ArgVec,
    template_args: &ArgVec,
    allow_array_fixup: bool,
) -> Result<(&'s str, DemangledArg), DemangleError<'s>> {
    if let Some(demangled) = demangle_qualifierless_arg(config, full_args)? {
        return Ok(demangled);
    }

    let Remaining {
        r: args,
        d: (sign, post_qualifiers),
    } = demangle_arg_qualifiers(full_args)?;
    let Remaining {
        r: args,
        d: (sign, post_qualifiers, array_qualifiers),
    } = demangle_array_pseudo_qualifier(config, args, sign, post_qualifiers, allow_array_fixup)?;

    if let Some(s) = args.strip_prefix('F') {
        let (r, fp) = demangle_function_pointer_arg(
            config,
            s,
            template_args,
            sign,
            post_qualifiers,
            array_qualifiers,
            allow_array_fixup,
        )?;
        Ok((r, DemangledArg::FunctionPointer(fp)))
    } else if let Some(r) = args.strip_prefix('M') {
        let (r, mp) = demangle_method_pointer_arg(
            config,
            r,
            full_args,
            template_args,
            sign,
            post_qualifiers,
            array_qualifiers,
            allow_array_fixup,
        )?;
        Ok((r, DemangledArg::MethodPointer(mp)))
    } else if let Some(r) = args.strip_prefix('O') {
        let (r, mp) = demangle_object_pointer_arg(
            config,
            r,
            full_args,
            template_args,
            sign,
            post_qualifiers,
            array_qualifiers,
            allow_array_fixup,
        )?;
        Ok((r, DemangledArg::Plain(mp, None.into())))
    } else {
        // 'G' is used for classes, structs and unions, so we must make sure we
        // don't parse a primitive type next, otherwise this is not properly
        // mangled.
        let (args, must_be_class_like) = args.c_maybe_strip_prefix('G');

        let Remaining {
            r,
            d: (is_class_like, typ, sign),
        } = demangle_arg_type(
            config,
            args,
            sign,
            parsed_arguments,
            template_args,
            allow_array_fixup,
        )?;

        if must_be_class_like && !is_class_like {
            return Err(DemangleError::PrimitiveInsteadOfClass(full_args));
        }

        let out = format!(
            "{}{}{}{}",
            sign,
            typ,
            if !post_qualifiers.is_empty() { " " } else { "" },
            post_qualifiers.trim_matches(' ')
        );

        Ok((r, DemangledArg::Plain(out, array_qualifiers)))
    }
}

fn demangle_arg_type<'s, 'pa, 't, 'out>(
    config: &DemangleConfig,
    args: &'s str,
    mut sign: Signedness,
    parsed_arguments: &'pa ArgVec,
    template_args: &'t ArgVec,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, (bool, Cow<'out, str>, Signedness)>, DemangleError<'s>>
where
    's: 'out,
    'pa: 'out,
    't: 'out,
{
    let c = args
        .chars()
        .next()
        .ok_or(DemangleError::RanOutOfArguments)?;

    let (args, is_class_like, typ) = match c {
        'c' => (&args[1..], false, Cow::from("char")),
        's' => (&args[1..], false, Cow::from("short")),
        'i' => (&args[1..], false, Cow::from("int")),
        'l' => (&args[1..], false, Cow::from("long")),
        'x' => (&args[1..], false, Cow::from("long long")),
        'f' => (&args[1..], false, Cow::from("float")),
        'd' => (&args[1..], false, Cow::from("double")),
        'r' => (&args[1..], false, Cow::from("long double")),
        'b' => (&args[1..], false, Cow::from("bool")),
        'w' => (&args[1..], false, Cow::from("wchar_t")),
        'v' => (&args[1..], false, Cow::from("void")),
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
                            "__uint128_t"
                        } else {
                            "__int128_t"
                        }
                    } else {
                        "int128_t"
                    }
                }
                _ => {
                    return Err(DemangleError::InvalidBitwidthForExtensionInteger(
                        args, bitwidth,
                    ));
                }
            };
            (r, false, Cow::from(typ))
        }
        '1'..='9' => {
            let Remaining { r, d: class_name } =
                demangle_custom_name(args, DemangleError::InvalidCustomNameOnArgument)?;
            (r, true, Cow::from(class_name))
        }
        'Q' => {
            let (remaining, namespaces, _trailing_namespace) =
                demangle_namespaces(config, &args[1..], template_args, allow_array_fixup)?;
            (remaining, true, Cow::from(namespaces))
        }
        'T' => {
            // Remembered type / look back
            let Remaining { r, d: lookback } = args[1..]
                .p_number_maybe_multi_digit()
                .ok_or(DemangleError::InvalidLookbackCount(args))?;

            let referenced_arg = parsed_arguments
                .get(lookback)
                .ok_or(DemangleError::LookbackCountTooBig(args, lookback))?;

            (r, false, Cow::from(referenced_arg))
        }
        't' => {
            // templates
            let (remaining, template, _typ) =
                demangle_template(config, &args[1..], template_args, allow_array_fixup)?;
            (remaining, true, Cow::from(template))
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

            (r, false, Cow::from(t))
        }
        _ => {
            return Err(DemangleError::UnknownType(c, args));
        }
    };

    Ok(Remaining::new(args, (is_class_like, typ, sign)))
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
    template_args: &ArgVec,
    sign: Signedness,
    post_qualifiers: String,
    array_qualifiers: OptionDisplay<ArrayQualifiers>,
    allow_array_fixup: bool,
) -> Result<(&'s str, FunctionPointer), DemangleError<'s>> {
    let (r, func_args) =
        demangle_argument_list_impl(config, s, None, template_args, true, allow_array_fixup)?;
    let Some(r) = r.strip_prefix('_') else {
        return Err(DemangleError::MissingReturnTypeForFunctionPointer(r));
    };

    let (r, return_type) =
        demangle_argument(config, r, &func_args, template_args, allow_array_fixup)?;

    let fp = match return_type {
        DemangledArg::Plain(plain, array_qualifiers) => FunctionPointer {
            return_type: format!("{sign}{plain}"),
            array_qualifiers,
            post_qualifiers,
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
        DemangledArg::Repeat { .. } | DemangledArg::Ellipsis => {
            return Err(DemangleError::InvalidReturnTypeForFunctionPointer(r))
        }
    };

    Ok((r, fp))
}

/// Method pointer
// TODO: fix too_many_arguments
#[expect(clippy::too_many_arguments)]
fn demangle_method_pointer_arg<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_args: &'s str,
    template_args: &ArgVec,
    sign: Signedness,
    post_qualifiers: String,
    array_qualifiers: OptionDisplay<ArrayQualifiers>,
    allow_array_fixup: bool,
) -> Result<(&'s str, MethodPointer), DemangleError<'s>> {
    if sign != Signedness::No || !post_qualifiers.chars().all(|c| c == '*') {
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

        let (r, fp) = demangle_function_pointer_arg(
            config,
            r,
            template_args,
            sign,
            post_qualifiers,
            array_qualifiers,
            allow_array_fixup,
        )?;
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
// TODO: fix too_many_arguments
#[expect(clippy::too_many_arguments)]
fn demangle_object_pointer_arg<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_args: &'s str,
    template_args: &ArgVec,
    sign: Signedness,
    post_qualifiers: String,
    array_qualifiers: OptionDisplay<ArrayQualifiers>,
    allow_array_fixup: bool,
) -> Result<(&'s str, String), DemangleError<'s>> {
    if sign != Signedness::No
        || !post_qualifiers.chars().all(|c| c == '*')
        || array_qualifiers.is_some()
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
            arg.push_str(&arr.inner_post_qualifiers);
        }
    }
    arg += &format!("({class_name}::{post_qualifiers})");
    if let Some(arr) = arr.as_option() {
        if !arr.inner_post_qualifiers.is_empty() {
            arg.push(')');
        }
        arg.push_str(&arr.arrays);
    }

    Ok((r, arg))
}

fn demangle_arg_qualifiers<'s>(
    s: &'s str,
) -> Result<Remaining<'s, (Signedness, String)>, DemangleError<'s>> {
    let mut remaining = s;
    let mut post_qualifiers = String::new();

    while !remaining.is_empty() {
        let Remaining { r, d: c } = remaining
            .p_first()
            .ok_or(DemangleError::RanOutOfArguments)?;

        match c {
            'P' => post_qualifiers.insert(0, '*'),
            'R' => post_qualifiers.insert(0, '&'),
            'C' => post_qualifiers.insert_str(0, "const "),
            'V' => post_qualifiers.insert_str(0, "volatile "),
            _ => break,
        }

        remaining = r;
    }

    // There can be at most one signedness qualifier as far as I know
    let (remaining, sign) = if let Some(Remaining { r, d: c }) = remaining.p_first() {
        match c {
            'S' => (r, Signedness::Signed),
            'U' => (r, Signedness::Unsigned),
            _ => (remaining, Signedness::No),
        }
    } else {
        (remaining, Signedness::No)
    };

    Ok(Remaining::new(remaining, (sign, post_qualifiers)))
}

// `allow_array_fixup` exists because array sizes are not always messed up.
// As far as I know, array sizes are correct only on templated functions.
fn demangle_array_pseudo_qualifier<'s>(
    config: &DemangleConfig,
    s: &'s str,
    mut sign: Signedness,
    mut post_qualifiers: String,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, (Signedness, String, OptionDisplay<ArrayQualifiers>)>, DemangleError<'s>>
{
    if !s.starts_with('A') {
        return Ok(Remaining::new(s, (sign, post_qualifiers, None.into())));
    }

    let mut array_qualifiers = ArrayQualifiers {
        inner_post_qualifiers: String::new(),
        arrays: String::new(),
    };

    if sign != Signedness::No {
        // Avoid stuff like "signed signed"
        return Err(DemangleError::PrevQualifiersInInvalidPostioniAtArrayArgument(s));
    }
    array_qualifiers.inner_post_qualifiers = post_qualifiers;

    let mut args = s;
    while let Some(remaining) = args.strip_prefix('A') {
        let Some(Remaining {
            r: remaining,
            d: array_length,
        }) = remaining.p_number()
        else {
            return Err(DemangleError::InvalidArraySize(remaining));
        };
        let Some(remaining) = remaining.strip_prefix('_') else {
            return Err(DemangleError::MalformedArrayArgumment(remaining));
        };

        let array_length = if config.fix_array_length_arg && allow_array_fixup {
            array_length + 1
        } else {
            array_length
        };

        array_qualifiers
            .arrays
            .push_str(&format!("[{array_length}]"));
        args = remaining;
    }

    let Remaining {
        r,
        d: (sign_other, post),
    } = demangle_arg_qualifiers(args)?;
    sign = sign_other;
    post_qualifiers = post;

    Ok(Remaining::new(
        r,
        (sign, post_qualifiers, Some(array_qualifiers).into()),
    ))
}
