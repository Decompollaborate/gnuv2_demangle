/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use core::num::NonZeroUsize;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::{DemangleConfig, DemangleError};

use crate::{
    remainer::{Remaining, StrParsing},
    str_cutter::StrCutter,
};

pub fn demangle<'s>(sym: &'s str, config: &DemangleConfig) -> Result<String, DemangleError<'s>> {
    if !sym.is_ascii() {
        return Err(DemangleError::NonAscii);
    }

    demangle_impl(sym, config, true)
}

fn demangle_impl<'s>(
    sym: &'s str,
    config: &DemangleConfig,
    allow_global_sym_keyed: bool,
) -> Result<String, DemangleError<'s>> {
    if let Some(s) = sym.strip_prefix("_$_") {
        demangle_destructor(config, s)
    } else if let Some(s) = sym.strip_prefix("__") {
        demangle_special(config, s, sym)
    } else if let Some(s) = sym.c_cond_and_strip_prefix(allow_global_sym_keyed, "_GLOBAL_$") {
        demangle_global_sym_keyed(config, s, sym)
    } else if let Some((func_name, args)) = sym.c_split2("__F") {
        demangle_free_function(config, func_name, args)
    } else if let Some((method_name, class_and_args)) =
        sym.c_split2_r_starts_with("__", |c| matches!(c, '1'..='9' | 'C' | 't' | 'H'))
    {
        demangle_method(config, method_name, class_and_args)
    } else if let Some((func_name, s)) = sym.c_split2("__Q") {
        demangle_namespaced_function(config, func_name, s)
    } else if let Some(sym) = sym.strip_prefix("_vt") {
        demangle_virtual_table(config, sym)
    } else if let Some((s, name)) = sym.c_split2("$") {
        demangle_namespaced_global(config, s, name)
    } else {
        Err(DemangleError::NotMangled)
    }
}

fn demangle_destructor<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if let Some(s) = s.strip_prefix('t') {
        let (remaining, template, typ) =
            demangle_template(config, s, &DemangledArgVec::new(config, None))?;

        if remaining.is_empty() {
            Ok(format!("{template}::~{typ}(void)"))
        } else {
            Err(DemangleError::TrailingDataOnDestructor(remaining))
        }
    } else if let Some(s) = s.strip_prefix('Q') {
        let (remaining, namespaces, trailing_namespace) =
            demangle_namespaces(config, s, &DemangledArgVec::new(config, None))?;

        if remaining.is_empty() {
            Ok(format!("{namespaces}::~{trailing_namespace}(void)"))
        } else {
            Err(DemangleError::TrailingDataOnDestructor(remaining))
        }
    } else {
        let Remaining { r, d: class_name } =
            demangle_custom_name(s, DemangleError::InvalidClassNameOnDestructor)?;

        if r.is_empty() {
            Ok(format!("{class_name}::~{class_name}(void)"))
        } else {
            Err(DemangleError::TrailingDataOnDestructor(r))
        }
    }
}

fn demangle_special<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_sym: &'s str,
) -> Result<String, DemangleError<'s>> {
    let c = s
        .chars()
        .next()
        .ok_or(DemangleError::RanOutWhileDemanglingSpecial)?;

    let (remaining, class_name, method_name, suffix) = if matches!(c, '1'..='9') {
        // class constructor
        let Remaining { r, d: class_name } =
            demangle_custom_name(s, DemangleError::InvalidClassNameOnConstructor)?;

        (r, Some(Cow::from(class_name)), Cow::from(class_name), "")
    } else if let Some(remaining) = s.strip_prefix("tf") {
        return demangle_type_info_function(config, remaining);
    } else if let Some(remaining) = s.strip_prefix("ti") {
        return demangle_type_info_node(config, remaining);
    } else if let Some(remaining) = s.strip_prefix('t') {
        let (remaining, template, typ) =
            demangle_template(config, remaining, &DemangledArgVec::new(config, None))?;

        (remaining, Some(Cow::from(template)), Cow::from(typ), "")
    } else if let Some(q_less) = s.strip_prefix('Q') {
        let (remaining, namespaces, trailing_namespace) =
            demangle_namespaces(config, q_less, &DemangledArgVec::new(config, None))?;

        (
            remaining,
            Some(Cow::from(namespaces)),
            Cow::from(trailing_namespace),
            "",
        )
    } else {
        let end_index = s.find("__").ok_or(DemangleError::InvalidSpecialMethod(s))?;
        let op = &s[..end_index];

        let remaining = &s[end_index + 2..];

        let method_name = match op {
            "nw" => Cow::from("operator new"),
            "dl" => Cow::from("operator delete"),
            "vn" => Cow::from("operator new []"),
            "eq" => Cow::from("operator=="),
            "ne" => Cow::from("operator!="),
            "as" => Cow::from("operator="),
            "vc" => Cow::from("operator[]"),
            "ad" => Cow::from("operator&"),
            "nt" => Cow::from("operator!"),
            "ls" => Cow::from("operator<<"),
            "rs" => Cow::from("operator>>"),
            "er" => Cow::from("operator^"),
            "lt" => Cow::from("operator<"),
            "aml" => Cow::from("operator*="),
            "apl" => Cow::from("operator+="),
            _ => {
                if let Some(cast) = op.strip_prefix("op") {
                    let (remaining, DemangledArg::Plain(typ)) = demangle_argument(
                        config,
                        cast,
                        &DemangledArgVec::new(config, None),
                        &DemangledArgVec::new(config, None),
                    )?
                    else {
                        return Err(DemangleError::UnrecognizedSpecialMethod(op));
                    };
                    if !remaining.is_empty() {
                        return Err(DemangleError::MalformedCastOperatorOverload(remaining));
                    }

                    Cow::from(format!("operator {typ}"))
                } else {
                    return {
                        // This may be a plain function that got confused with a
                        // special symbol, so try to decode as a function instead.
                        if let Some((func_name, args)) = full_sym.c_split2("__F") {
                            demangle_free_function(config, func_name, args)
                        } else if let Some((incomplete_method_name, class_and_args)) = s
                            .c_split2_r_starts_with("__", |c| {
                                matches!(c, '1'..='9' | 'C' | 't' | 'H')
                            })
                        {
                            // split `s` instead of `full_sym` to skip over the
                            // first `__`,
                            // if that check passes, then recover the actual
                            // method name, including the initial `__`, by
                            // using the length of the `incomplete_method_name`
                            // to slice the `full_sym`.

                            let method_name = &full_sym[..incomplete_method_name.len() + 2];
                            demangle_method(config, method_name, class_and_args)
                        } else {
                            Err(DemangleError::UnrecognizedSpecialMethod(op))
                        }
                    };
                }
            }
        };

        if let Some(remaining) = remaining.strip_prefix('F') {
            (remaining, None, method_name, "")
        } else {
            let Remaining {
                r: remaining,
                d: suffix,
            } = demangle_method_qualifier(remaining);

            let (remaining, namespaces) = if let Some(q_less) = remaining.strip_prefix('Q') {
                let (remaining, namespaces, _trailing_namespace) =
                    demangle_namespaces(config, q_less, &DemangledArgVec::new(config, None))?;

                (remaining, Cow::from(namespaces))
            } else if let Some(r) = remaining.strip_prefix('t') {
                let (remaining, template, _typ) =
                    demangle_template(config, r, &DemangledArgVec::new(config, None))?;

                (remaining, Cow::from(template))
            } else {
                let Remaining { r, d: class_name } =
                    demangle_custom_name(remaining, DemangleError::InvalidClassNameOnOperator)?
                        .d_as_cow();

                (r, class_name)
            };

            (remaining, Some(namespaces), method_name, suffix)
        }
    };

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(
            config,
            remaining,
            class_name.as_deref(),
            &DemangledArgVec::new(config, None),
        )?
    };

    let out = if let Some(class_name) = class_name {
        format!("{class_name}::{method_name}({argument_list}){suffix}")
    } else {
        format!("{method_name}({argument_list}){suffix}")
    };
    Ok(out)
}

fn demangle_free_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let argument_list =
        demangle_argument_list(config, args, None, &DemangledArgVec::new(config, None))?;

    Ok(format!("{func_name}({argument_list})"))
}

fn demangle_argument_list<'s>(
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

fn demangle_argument_list_impl<'c, 's, 'ns>(
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

fn demangle_method<'s>(
    config: &DemangleConfig,
    method_name: &'s str,
    class_and_args: &'s str,
) -> Result<String, DemangleError<'s>> {
    let Remaining {
        r: remaining,
        d: suffix,
    } = demangle_method_qualifier(class_and_args);

    let (remaining, namespace) = if let Some(templated) = remaining.strip_prefix('t') {
        let (remaining, template, _typ) =
            demangle_template(config, templated, &DemangledArgVec::new(config, None))?;

        (remaining, Cow::from(template))
    } else if let Some(q_less) = remaining.strip_prefix('Q') {
        let (remaining, namespaces, _trailing_namespace) =
            demangle_namespaces(config, q_less, &DemangledArgVec::new(config, None))?;

        (remaining, Cow::from(namespaces))
    } else if let Some(with_return_type) = remaining.strip_prefix('H') {
        let (remaining, template_args, typ) =
            demangle_template_with_return_type(config, with_return_type)?;

        let (remaining, typ) = if let Some(typ) = typ {
            (remaining, Some(typ))
        } else if let Some(r) = remaining.strip_prefix('t') {
            let (r, template, _typ) =
                demangle_template(config, r, &DemangledArgVec::new(config, None))?;

            (r, Some(Cow::from(template)))
        } else {
            (remaining, None)
        };

        let (remaining, argument_list) =
            demangle_argument_list_impl(config, remaining, typ.as_deref(), &template_args, false)?;
        let prefix = if let Some(r) = remaining.strip_prefix('_') {
            let (r, DemangledArg::Plain(mut ret_type)) = demangle_argument(
                config,
                r,
                &DemangledArgVec::new(config, typ.as_deref()),
                &template_args,
            )?
            else {
                return Err(DemangleError::MalformedTemplateWithReturnTypeMissingReturnType(r));
            };
            if !r.is_empty() {
                return Err(
                    DemangleError::TrailingDataAfterReturnTypeOfMalformedTemplateWithReturnType(r),
                );
            }
            ret_type.push(' ');
            ret_type
        } else {
            return Err(DemangleError::MalformedTemplateWithReturnTypeMissingReturnType(remaining));
        };

        let template_args = template_args.join();
        let formated_template_args = if template_args.ends_with('>') {
            format!("<{} >", template_args)
        } else {
            format!("<{}>", template_args)
        };
        let argument_list = argument_list.join();
        return Ok(if let Some(typ) = typ {
            format!("{prefix}{typ}::{method_name}{formated_template_args}({argument_list}){suffix}")
        } else {
            format!("{prefix}{method_name}{formated_template_args}({argument_list}){suffix}")
        });
    } else {
        let Remaining { r, d: class_name } =
            demangle_custom_name(remaining, DemangleError::InvalidClassNameOnMethod)?.d_as_cow();

        (r, class_name)
    };

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(
            config,
            remaining,
            Some(&namespace),
            &DemangledArgVec::new(config, None),
        )?
    };

    Ok(format!(
        "{namespace}::{method_name}({argument_list}){suffix}"
    ))
}

fn demangle_namespaced_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, namespaces, _trailing_namespace) =
        demangle_namespaces(config, s, &DemangledArgVec::new(config, None))?;

    let argument_list = if remaining.is_empty() {
        "void"
    } else {
        &demangle_argument_list(
            config,
            remaining,
            Some(&namespaces),
            &DemangledArgVec::new(config, None),
        )?
    };

    let out = format!("{namespaces}::{func_name}({argument_list})");
    Ok(out)
}

fn demangle_type_info_function<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if let (remaining, DemangledArg::Plain(demangled_type)) = demangle_argument(
        config,
        s,
        &DemangledArgVec::new(config, None),
        &DemangledArgVec::new(config, None),
    )? {
        if remaining.is_empty() {
            Ok(format!("{demangled_type} type_info function"))
        } else {
            Err(DemangleError::TrailingDataOnTypeInfoFunction(remaining))
        }
    } else {
        Err(DemangleError::InvalidTypeOnTypeInfoFunction(s))
    }
}

fn demangle_type_info_node<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    if let (remaining, DemangledArg::Plain(demangled_type)) = demangle_argument(
        config,
        s,
        &DemangledArgVec::new(config, None),
        &DemangledArgVec::new(config, None),
    )? {
        if remaining.is_empty() {
            Ok(format!("{demangled_type} type_info node"))
        } else {
            Err(DemangleError::TrailingDataOnTypeInfoNode(remaining))
        }
    } else {
        Err(DemangleError::InvalidTypeOnTypeInfoNode(s))
    }
}

fn demangle_virtual_table<'s>(
    config: &DemangleConfig,
    s: &'s str,
) -> Result<String, DemangleError<'s>> {
    let mut remaining = s;
    let mut stuff = Vec::new();

    while !remaining.is_empty() {
        remaining = remaining
            .strip_prefix('$')
            .ok_or(DemangleError::VTableMissingDollarSeparator(remaining))?;

        remaining = if let Some(r) = remaining.strip_prefix('t') {
            let (r, template, _typ) =
                demangle_template(config, r, &DemangledArgVec::new(config, None))?;

            stuff.push(Cow::from(template));
            r
        } else if let Some(r) = remaining.strip_prefix('Q') {
            let (r, namespaces, _trailing_namespace) =
                demangle_namespaces(config, r, &DemangledArgVec::new(config, None))?;

            stuff.push(Cow::from(namespaces));
            r
        } else {
            let Remaining { r, d: class_name } =
                demangle_custom_name(remaining, DemangleError::InvalidClassNameOnVirtualTable)?
                    .d_as_cow();

            stuff.push(class_name);
            r
        };
    }

    Ok(format!("{} virtual table", stuff.join("::")))
}

fn demangle_namespaced_global<'s>(
    config: &DemangleConfig,
    s: &'s str,
    name: &'s str,
) -> Result<String, DemangleError<'s>> {
    let Some(remaining) = s.strip_prefix('_') else {
        return Err(DemangleError::InvalidNamespacedGlobal(s, name));
    };

    let (r, space) = if let Some(r) = remaining.strip_prefix('t') {
        let (r, template, _typ) =
            demangle_template(config, r, &DemangledArgVec::new(config, None))?;

        (r, Cow::from(template))
    } else if let Some(r) = remaining.strip_prefix('Q') {
        let (r, namespaces, _trailing_namespace) =
            demangle_namespaces(config, r, &DemangledArgVec::new(config, None))?;

        (r, Cow::from(namespaces))
    } else {
        let Remaining { r, d: class_name } =
            demangle_custom_name(remaining, DemangleError::InvalidNamespaceOnNamespacedGlobal)?
                .d_as_cow();

        (r, class_name)
    };

    if !r.is_empty() {
        return Err(DemangleError::TrailingDataOnNamespacedGlobal(r));
    }

    Ok(format!("{space}::{name}"))
}

fn demangle_global_sym_keyed<'s>(
    config: &DemangleConfig,
    s: &'s str,
    full_sym: &'s str,
) -> Result<String, DemangleError<'s>> {
    let (remaining, which, is_constructor) = if let Some(r) = s.strip_prefix("I$") {
        (r, "constructors", true)
    } else if let Some(r) = s.strip_prefix("D$") {
        (r, "destructors", false)
    } else if let Some(r) = s.strip_prefix("F$") {
        if config.demangle_global_keyed_frames {
            (r, "frames", false)
        } else {
            // !HACK(c++filt): c++filt does not recognize `_GLOBAL_$F$`, so it
            // !tries to demangle it as anything else.
            return demangle_impl(full_sym, config, false);
        }
    } else {
        return Err(DemangleError::InvalidGlobalSymKeyed(s));
    };

    let demangled_sym = demangle_impl(remaining, config, false);
    if config.preserve_namespaced_global_constructor_bug
        && is_constructor
        && remaining.starts_with("__Q")
    {
        // !HACK(c++filt): Seems like c++filt has a bug where it won't output
        // !the "global constructors keyed to " prefix for namespaced functions
        return demangled_sym;
    }

    let actual_sym = demangled_sym
        .map(Cow::from)
        .unwrap_or_else(|_| Cow::from(remaining));

    Ok(format!("global {which} keyed to {actual_sym}"))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum DemangledArg {
    Plain(String),
    Repeat { count: NonZeroUsize, index: usize },
    Ellipsis,
}

struct DemangledArgVec<'c, 'ns> {
    _config: &'c DemangleConfig,
    namespace: Option<&'ns str>,
    args: Vec<DemangledArg>,
    trailing_ellipsis: bool,
}

impl<'c, 'ns> DemangledArgVec<'c, 'ns> {
    fn new(config: &'c DemangleConfig, namespace: Option<&'ns str>) -> Self {
        Self {
            _config: config,
            namespace,
            args: Vec::new(),
            trailing_ellipsis: false,
        }
    }

    fn get(&self, mut index: usize) -> Option<&str> {
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

    fn push<'s>(
        &mut self,
        arg: DemangledArg,
        s: &'s str,
        remaining: &'s str,
        allow_data_after_ellipsis: bool,
    ) -> Result<bool, DemangleError<'s>> {
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
                self.trailing_ellipsis = true;
                return Ok(true);
            }
        };
        self.args.push(arg);
        Ok(false)
    }

    fn join(self) -> String {
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
            // !HACK(c++filt):  Special case to mimic c++filt, since it doesn't
            // !use an space between the comma and the ellipsis.
            out.push_str(",...");
        }
        out
    }
}

fn demangle_argument<'s>(
    config: &DemangleConfig,
    full_args: &'s str,
    parsed_arguments: &DemangledArgVec,
    template_args: &DemangledArgVec,
) -> Result<(&'s str, DemangledArg), DemangleError<'s>> {
    if let Some(repeater) = full_args.strip_prefix('N') {
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
        return Ok((remaining, DemangledArg::Repeat { count, index }));
    } else if let Some(remaining) = full_args.strip_prefix('e') {
        return Ok((remaining, DemangledArg::Ellipsis));
    } else if let Some(remaining) = full_args.strip_prefix('t') {
        let (remaining, template, _typ) = demangle_template(config, remaining, template_args)?;

        return Ok((remaining, DemangledArg::Plain(template)));
    }

    let mut args = full_args;

    let Remaining {
        r,
        d: (mut pre_qualifier, mut post_qualifiers),
    } = demangle_arg_qualifiers(args)?;
    args = r;

    if args.starts_with('A') {
        // Avoid stuff like "signed signed"
        if !pre_qualifier.is_empty() {
            return Err(DemangleError::PrevQualifiersInInvalidPostioniAtArrayArgument(full_args));
        }
        post_qualifiers.insert(0, '(');
        post_qualifiers.push(')');

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
        args = r;
    }
    let pre_qualifier = pre_qualifier;
    let post_qualifiers = post_qualifiers;

    // 'G' is used for classes, structs and unions, so we must make sure we
    // don't parse a primitive type next, otherwise this is not properly
    // mangled.
    let must_be_class_like = if let Some(a) = args.strip_prefix('G') {
        args = a;
        true
    } else {
        false
    };

    let c = args
        .chars()
        .next()
        .ok_or(DemangleError::RanOutOfArguments)?;
    let mut is_class_like = false;
    // Plain types
    let typ = match c {
        'c' => Cow::from("char"),
        's' => Cow::from("short"),
        'i' => Cow::from("int"),
        'l' => Cow::from("long"),
        'x' => Cow::from("long long"),
        'f' => Cow::from("float"),
        'd' => Cow::from("double"),
        'r' => Cow::from("long double"),
        'b' => Cow::from("bool"),
        'w' => Cow::from("wchar_t"),
        'v' => Cow::from("void"),
        '1'..='9' => {
            let Remaining { r, d: class_name } =
                demangle_custom_name(args, DemangleError::InvalidCustomNameOnArgument)?;
            args = r;
            is_class_like = true;
            Cow::from(class_name)
        }
        'Q' => {
            let (remaining, namespaces, _trailing_namespace) =
                demangle_namespaces(config, &args[1..], template_args)?;
            args = remaining;
            is_class_like = true;
            Cow::from(namespaces)
        }
        'T' => {
            // Remembered type / look back
            let Remaining {
                r: remaining,
                d: lookback,
            } = args[1..]
                .p_number_maybe_multi_digit()
                .ok_or(DemangleError::InvalidLookbackCount(args))?;

            let referenced_arg = parsed_arguments
                .get(lookback)
                .ok_or(DemangleError::LookbackCountTooBig(args, lookback))?;

            args = remaining;

            // Not really, since lookback could reference anything...
            is_class_like = true;

            Cow::from(referenced_arg)
        }
        // TODO :is this dead code?
        't' => {
            // templates
            let (remaining, template, _typ) = demangle_template(config, &args[1..], template_args)?;

            args = remaining;

            is_class_like = true;
            Cow::from(template)
        }
        'F' => {
            // Function pointer/reference
            let (r, subargs) = demangle_argument_list_impl(
                config,
                &args[1..],
                None,
                &DemangledArgVec::new(config, None),
                true,
            )?;

            let Some(r) = r.strip_prefix('_') else {
                return Err(DemangleError::MissingReturnTypeForFunctionPointer(r));
            };

            let (r, DemangledArg::Plain(ret)) =
                demangle_argument(config, r, &subargs, &DemangledArgVec::new(config, None))?
            else {
                return Err(DemangleError::InvalidReturnTypeForFunctionPointer(r));
            };

            return Ok((
                r,
                DemangledArg::Plain(format!(
                    "{}{}{}({})({})",
                    pre_qualifier,
                    ret,
                    if ret.ends_with(['*', '&']) { "" } else { " " },
                    post_qualifiers.trim_matches(' '),
                    subargs.join()
                )),
            ));
        }
        'X' => {
            args = &args[1..];
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

            args = r;
            is_class_like = true;

            Cow::from(t)
        }
        _ => {
            return Err(DemangleError::UnknownType(c, args));
        }
    };

    if must_be_class_like && !is_class_like {
        return Err(DemangleError::PrimitiveInsteadOfClass(full_args));
    }

    if !is_class_like {
        args = &args[1..];
    }

    let out = format!(
        "{}{}{}{}",
        pre_qualifier,
        typ,
        if !post_qualifiers.is_empty() { " " } else { "" },
        post_qualifiers.trim_matches(' ')
    );

    Ok((args, DemangledArg::Plain(out)))
}

// 'Q' must be stripped already
fn demangle_namespaces<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &DemangledArgVec,
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let Remaining {
        r: remaining,
        d: namespace_count,
    } = if let Some(r) = s.strip_prefix('_') {
        // More than a single digit of namespaces
        r.p_number().and_then(|Remaining { r, d }| {
            r.strip_prefix('_').map(|new_r| Remaining::new(new_r, d))
        })
    } else {
        s.p_digit()
    }
    .ok_or(DemangleError::InvalidNamespaceCount(s))?;

    let namespace_count =
        NonZeroUsize::new(namespace_count).ok_or(DemangleError::InvalidNamespaceCount(s))?;

    demangle_namespaces_impl(config, remaining, namespace_count, template_args)
}

fn demangle_namespaces_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespace_count: NonZeroUsize,
    template_args: &DemangledArgVec,
) -> Result<(&'s str, String, &'s str), DemangleError<'s>> {
    let mut namespaces = String::new();
    let mut remaining = s;
    let mut trailing_type = "";

    for _ in 0..namespace_count.get() {
        if !namespaces.is_empty() {
            namespaces.push_str("::");
        }

        let (r, n) = if let Some(temp) = remaining.strip_prefix('t') {
            let (r, template, typ) = demangle_template(config, temp, template_args)?;
            trailing_type = typ;
            (r, Cow::from(template))
        } else {
            let Remaining { r, d: ns } =
                demangle_custom_name(remaining, DemangleError::InvalidCustomNameOnNamespace)?;
            trailing_type = ns;
            (r, Cow::from(ns))
        };
        remaining = r;
        namespaces.push_str(&n);
    }

    Ok((remaining, namespaces, trailing_type))
}

fn demangle_template<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &DemangledArgVec,
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

// TODO: fix this
#[allow(clippy::type_complexity)]
fn demangle_template_with_return_type<'c, 's>(
    config: &'c DemangleConfig,
    s: &'s str,
) -> Result<(&'s str, DemangledArgVec<'c, 's>, Option<Cow<'s, str>>), DemangleError<'s>> {
    let remaining = s;
    let Some(Remaining {
        r: remaining,
        d: digit,
    }) = remaining.p_digit()
    else {
        return Err(DemangleError::InvalidTemplateReturnCount(s));
    };
    let digit = NonZeroUsize::new(digit).ok_or(DemangleError::TemplateReturnCountIsZero(s))?;

    let (remaining, types) = demangle_template_types_impl(
        config,
        remaining,
        digit,
        &DemangledArgVec::new(config, None),
    )?;

    let Some(remaining) = remaining.strip_prefix('_') else {
        return Err(DemangleError::MalformedTemplateWithReturnType(remaining));
    };
    let (remaining, namespaces) = if let Some(q_less) = remaining.strip_prefix('Q') {
        let (r, namespaces, _trailing_namespace) =
            demangle_namespaces(config, q_less, &DemangledArgVec::new(config, None))?;

        (r, Some(Cow::from(namespaces)))
    } else if remaining.starts_with(|c| matches!(c, '1'..='9')) {
        let Remaining { r, d: namespace } = demangle_custom_name(
            remaining,
            DemangleError::InvalidNamespaceOnTemplatedFunction,
        )?
        .d_as_cow();
        (r, Some(namespace))
    } else {
        (remaining, None)
    };

    Ok((remaining, types, namespaces))
}

fn demangle_template_types_impl<'c, 's>(
    config: &'c DemangleConfig,
    s: &'s str,
    count: NonZeroUsize,
    template_args: &DemangledArgVec,
) -> Result<(&'s str, DemangledArgVec<'c, 's>), DemangleError<'s>> {
    let mut remaining = s;

    let mut types = DemangledArgVec::new(config, None);

    for _ in 0..count.get() {
        let r = if let Some(r) = remaining.strip_prefix('Z') {
            let old_args = r;
            let (r, arg) = demangle_argument(config, old_args, &types, template_args)?;

            if types.push(arg, old_args, r, false)? {
                remaining = r;
                break;
            }
            r
        } else {
            let mut r = remaining;
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

            if is_pointer || is_reference {
                let (aux, DemangledArg::Plain(_arg)) = demangle_argument(
                    config,
                    r,
                    &DemangledArgVec::new(config, None),
                    &DemangledArgVec::new(config, None),
                )?
                else {
                    return Err(DemangleError::InvalidTemplatedPointerReferenceValue(r));
                };
                let Remaining { r: aux, d: symbol } =
                    demangle_custom_name(aux, DemangleError::InvalidSymbolNameOnTemplateType)?;
                let t = format!("{}{}", if is_pointer { "&" } else { "" }, symbol);
                types.push(DemangledArg::Plain(t), r, aux, false)?;
                aux
            } else {
                let c = r.chars().next().ok_or(DemangleError::RanOutOfArguments)?;
                r = &r[1..];

                match c {
                    // "char" | "wchar_t"
                    'c' | 'w' => {
                        let Remaining { r, d: number } = r
                            .p_number()
                            .ok_or(DemangleError::InvalidTemplatedNumberForCharacterValue(r))?;
                        let demangled_char = char::from_u32(number.try_into().map_err(|_| {
                            DemangleError::InvalidTemplatedCharacterValue(r, number)
                        })?)
                        .ok_or(DemangleError::InvalidTemplatedCharacterValue(r, number))?;
                        let t = format!("'{demangled_char}'");
                        types.push(DemangledArg::Plain(t), r, r, false)?;
                        r
                    }
                    // "short" | "int" | "long" | "long long"
                    's' | 'i' | 'l' | 'x' => {
                        let (r, negative) = if let Some(r) = r.strip_prefix('m') {
                            (r, true)
                        } else {
                            (r, false)
                        };
                        let Remaining { r, d: number } = r
                            .p_number()
                            .ok_or(DemangleError::InvalidValueForIntegralTemplated(r))?;
                        let t = format!("{}{}", if negative { "-" } else { "" }, number);
                        types.push(DemangledArg::Plain(t), r, r, false)?;
                        r
                    }
                    // 'f' => {}, // "float"
                    // 'd' => {}, // "double"
                    // 'r' => {}, // "long double"
                    // "bool"
                    'b' => match r.chars().next() {
                        Some('1') => {
                            types.push(DemangledArg::Plain("true".to_string()), r, r, false)?;
                            &r[1..]
                        }
                        Some('0') => {
                            types.push(DemangledArg::Plain("false".to_string()), r, r, false)?;
                            &r[1..]
                        }
                        _ => return Err(DemangleError::InvalidTemplatedBoolean(r)),
                    },
                    _ => return Err(DemangleError::InvalidTypeValueForTemplated(c, r)),
                }
            }
        };

        remaining = r;
    }

    Ok((remaining, types))
}

fn demangle_custom_name<'s, F>(
    s: &'s str,
    err: F,
) -> Result<Remaining<'s, &'s str>, DemangleError<'s>>
where
    F: Fn(&'s str) -> DemangleError<'s>,
{
    let Remaining { r, d: length } = s.p_number().ok_or_else(|| err(s))?;

    if r.len() < length {
        Err(err(s))
    } else {
        Ok(Remaining::split_at(r, length))
    }
}

fn demangle_method_qualifier(s: &str) -> Remaining<'_, &str> {
    if let Some(remaining) = s.strip_prefix('C') {
        Remaining::new(remaining, " const")
    } else {
        Remaining::new(s, "")
    }
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
