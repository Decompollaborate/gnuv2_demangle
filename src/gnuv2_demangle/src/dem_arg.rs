/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroUsize;

use alloc::{borrow::Cow, string::String, vec::Vec};

use crate::str_cutter::StrCutter;
use crate::{DemangleConfig, DemangleError};

use crate::{
    dem::demangle_custom_name,
    dem_namespace::demangle_namespaces,
    dem_template::demangle_template,
    remainer::{Remaining, StrParsing},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DemangledArg {
    Plain(String),
    Repeat { count: NonZeroUsize, index: usize },
    Ellipsis,
}

pub(crate) struct DemangledArgVec<'c, 'ns> {
    config: &'c DemangleConfig,
    namespace: Option<&'ns str>,
    args: Vec<DemangledArg>,

    /// !HACK(c++filt): Allows to avoid emitting an space between a comma and
    /// the ellipsis.
    /// This is will always be `false` if `DemangleConfig::ellipsis_emit_space_after_comma`
    /// is set to `true`. Ellipsis will be handled as yet another element
    /// inside the `args` vector.
    trailing_ellipsis: bool,
}

impl<'c, 'ns> DemangledArgVec<'c, 'ns> {
    pub(crate) fn new(config: &'c DemangleConfig, namespace: Option<&'ns str>) -> Self {
        Self {
            config,
            namespace,
            args: Vec::new(),
            trailing_ellipsis: false,
        }
    }

    pub(crate) fn get(&self, mut index: usize) -> Option<&str> {
        if let Some(namespace) = self.namespace {
            if index == 0 {
                return Some(namespace);
            } else {
                index -= 1;
            }
        }

        loop {
            let arg = self.args.get(index)?;
            match arg {
                DemangledArg::Plain(p) => break Some(p),
                DemangledArg::Repeat { count: _, index: i } => {
                    if *i >= index {
                        break None;
                    }
                    index = *i;
                }
                DemangledArg::Ellipsis => break Some("..."),
            }
        }
    }

    pub(crate) fn push<'s>(
        &mut self,
        arg: DemangledArg,
        s: &'s str,
        remaining: &'s str,
        allow_data_after_ellipsis: bool,
    ) -> Result<bool, DemangleError<'s>> {
        let mut found_end = false;

        let arg = match arg {
            a @ DemangledArg::Plain(_) => a,
            DemangledArg::Repeat { count, index } => {
                // Check the index is in-bounds
                if self.namespace.is_some() {
                    if index == 0 {
                        // we're grood
                    } else {
                        let i = index - 1;
                        if i >= self.args.len() {
                            return Err(DemangleError::InvalidRepeatingArgument(s));
                        }
                    }
                } else if index >= self.args.len() {
                    return Err(DemangleError::InvalidRepeatingArgument(s));
                }

                let one = NonZeroUsize::new(1).expect("One will never be zero. Crazy, right?");
                for _ in 0..count.get() - 1 {
                    self.args.push(DemangledArg::Repeat { count: one, index });
                }
                DemangledArg::Repeat { count: one, index }
            }
            DemangledArg::Ellipsis => {
                if !allow_data_after_ellipsis && !remaining.is_empty() {
                    return Err(DemangleError::TrailingDataAfterEllipsis(remaining));
                }
                found_end = true;
                if !self.config.ellipsis_emit_space_after_comma {
                    self.trailing_ellipsis = true;
                    return Ok(found_end);
                }
                DemangledArg::Ellipsis
            }
        };
        self.args.push(arg);
        Ok(found_end)
    }

    pub(crate) fn join(self) -> String {
        let mut args = Vec::with_capacity(self.args.len());

        for arg in &self.args {
            match arg {
                DemangledArg::Plain(plain) => args.push(plain.as_str()),
                DemangledArg::Repeat { count, index } => {
                    let arg = if let Some(namespace) = self.namespace {
                        if *index == 0 {
                            namespace
                        } else {
                            args.get(*index - 1)
                                .expect("Indices were verified when pushing the arguments")
                        }
                    } else {
                        args.get(*index)
                            .expect("Indices were verified when pushing the arguments")
                    };

                    for _ in 0..count.get() {
                        args.push(arg);
                    }
                }
                DemangledArg::Ellipsis => args.push("..."),
            }
        }

        let mut out = args.join(", ");
        if self.trailing_ellipsis {
            // !HACK(c++filt): Special case to mimic c++filt, since it doesn't
            // !use an space between the comma and the ellipsis.
            if !out.is_empty() {
                out.push(',');
            }
            out.push_str("...");
        }
        out
    }
}

pub(crate) fn demangle_argument_list<'s>(
    config: &DemangleConfig,
    args: &'s str,
    namespace: Option<&str>,
    template_args: &DemangledArgVec,
) -> Result<String, DemangleError<'s>> {
    let (remaining, argument_list) =
        demangle_argument_list_impl(config, args, namespace, template_args, false)?;

    if !remaining.is_empty() {
        return Err(DemangleError::TrailingDataAfterArgumentList(remaining));
    }

    Ok(argument_list.join())
}

pub(crate) fn demangle_argument_list_impl<'c, 's, 'ns>(
    config: &'c DemangleConfig,
    mut args: &'s str,
    namespace: Option<&'ns str>,
    template_args: &DemangledArgVec,
    allow_data_after_ellipsis: bool,
) -> Result<(&'s str, DemangledArgVec<'c, 'ns>), DemangleError<'s>> {
    let mut arguments = DemangledArgVec::new(config, namespace);

    while !args.is_empty() && !args.starts_with('_') {
        let old_args = args;
        let (remaining, b) = demangle_argument(config, old_args, &arguments, template_args)?;

        args = remaining;
        if arguments.push(b, old_args, remaining, allow_data_after_ellipsis)? {
            break;
        }
    }

    Ok((args, arguments))
}

pub(crate) fn demangle_argument<'s>(
    config: &DemangleConfig,
    full_args: &'s str,
    parsed_arguments: &DemangledArgVec,
    template_args: &DemangledArgVec,
) -> Result<(&'s str, DemangledArg), DemangleError<'s>> {
    if let Some(demangled) = demangle_qualifierless_arg(config, full_args)? {
        return Ok(demangled);
    }

    let Remaining {
        r: args,
        d: (pre_qualifier, post_qualifiers),
    } = demangle_arg_qualifiers(full_args)?;
    let Remaining {
        r: args,
        d: (pre_qualifier, post_qualifiers),
    } = demangle_array_pseudo_qualifier(config, args, pre_qualifier, post_qualifiers)?;

    if let Some(s) = args.strip_prefix('F') {
        return demangle_function_pointer_arg(config, s, pre_qualifier, &post_qualifiers);
    }

    // 'G' is used for classes, structs and unions, so we must make sure we
    // don't parse a primitive type next, otherwise this is not properly
    // mangled.
    let (args, must_be_class_like) = args.c_maybe_strip_prefix('G');

    let Remaining {
        r,
        d: (is_class_like, typ),
    } = demangle_arg_type(config, args, parsed_arguments, template_args)?;

    if must_be_class_like && !is_class_like {
        return Err(DemangleError::PrimitiveInsteadOfClass(full_args));
    }

    let out = format!(
        "{}{}{}{}",
        pre_qualifier,
        typ,
        if !post_qualifiers.is_empty() { " " } else { "" },
        post_qualifiers.trim_matches(' ')
    );

    Ok((r, DemangledArg::Plain(out)))
}

fn demangle_arg_type<'s, 'pa, 't, 'out>(
    config: &DemangleConfig,
    args: &'s str,
    parsed_arguments: &'pa DemangledArgVec,
    template_args: &'t DemangledArgVec,
) -> Result<Remaining<'s, (bool, Cow<'out, str>)>, DemangleError<'s>>
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
        '1'..='9' => {
            let Remaining { r, d: class_name } =
                demangle_custom_name(args, DemangleError::InvalidCustomNameOnArgument)?;
            (r, true, Cow::from(class_name))
        }
        'Q' => {
            let (remaining, namespaces, _trailing_namespace) =
                demangle_namespaces(config, &args[1..], template_args)?;
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
            let (remaining, template, _typ) = demangle_template(config, &args[1..], template_args)?;
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

    Ok(Remaining::new(args, (is_class_like, typ)))
}

/// Handles any arg that can't be qualified
fn demangle_qualifierless_arg<'s>(
    _config: &DemangleConfig,
    full_args: &'s str,
) -> Result<Option<(&'s str, DemangledArg)>, DemangleError<'s>> {
    #[allow(clippy::manual_map)]
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
    pre_qualifier: &str,
    post_qualifiers: &str,
) -> Result<(&'s str, DemangledArg), DemangleError<'s>> {
    let (r, subargs) =
        demangle_argument_list_impl(config, s, None, &DemangledArgVec::new(config, None), true)?;
    let Some(r) = r.strip_prefix('_') else {
        return Err(DemangleError::MissingReturnTypeForFunctionPointer(r));
    };

    let (r, DemangledArg::Plain(ret)) =
        demangle_argument(config, r, &subargs, &DemangledArgVec::new(config, None))?
    else {
        return Err(DemangleError::InvalidReturnTypeForFunctionPointer(r));
    };

    let out = format!(
        "{}{}{}({})({})",
        pre_qualifier,
        ret,
        if ret.ends_with(['*', '&']) { "" } else { " " },
        post_qualifiers.trim_matches(' '),
        subargs.join()
    );
    Ok((r, DemangledArg::Plain(out)))
}

fn demangle_arg_qualifiers<'s>(
    s: &'s str,
) -> Result<Remaining<'s, (&'s str, String)>, DemangleError<'s>> {
    let mut remaining = s;
    let mut pre_qualifier = "";
    let mut post_qualifiers = String::new();

    while !remaining.is_empty() {
        let Remaining { r, d: c } = remaining
            .p_first()
            .ok_or(DemangleError::RanOutOfArguments)?;

        match c {
            'P' => post_qualifiers.insert(0, '*'),
            'R' => post_qualifiers.insert(0, '&'),
            'C' => post_qualifiers.insert_str(0, "const "),
            'S' => {
                if pre_qualifier.is_empty() {
                    pre_qualifier = "signed "
                } else {
                    return Err(DemangleError::FoundDuplicatedPrevQualifierOnArgument(s, c));
                }
            }
            'U' => {
                if pre_qualifier.is_empty() {
                    pre_qualifier = "unsigned "
                } else {
                    return Err(DemangleError::FoundDuplicatedPrevQualifierOnArgument(s, c));
                }
            }
            _ => break,
        }

        remaining = r;
    }

    Ok(Remaining::new(remaining, (pre_qualifier, post_qualifiers)))
}

fn demangle_array_pseudo_qualifier<'s>(
    config: &DemangleConfig,
    s: &'s str,
    mut pre_qualifier: &'s str,
    mut post_qualifiers: String,
) -> Result<Remaining<'s, (&'s str, String)>, DemangleError<'s>> {
    let r = if s.starts_with('A') {
        if !pre_qualifier.is_empty() {
            // Avoid stuff like "signed signed"
            return Err(DemangleError::PrevQualifiersInInvalidPostioniAtArrayArgument(s));
        }
        post_qualifiers.insert(0, '(');
        post_qualifiers.push(')');

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

            let array_length = if config.fix_array_length_arg {
                array_length + 1
            } else {
                array_length
            };

            post_qualifiers.push_str(&format!("[{array_length}]"));
            args = remaining;
        }

        let Remaining { r, d: (pre, post) } = demangle_arg_qualifiers(args)?;
        pre_qualifier = pre;
        post_qualifiers = post + &post_qualifiers;
        r
    } else {
        s
    };

    Ok(Remaining::new(r, (pre_qualifier, post_qualifiers)))
}
