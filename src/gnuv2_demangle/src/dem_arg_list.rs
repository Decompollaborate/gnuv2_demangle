/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::{DemangleConfig, DemangleError};

use crate::dem_arg::{demangle_argument, DemangledArg};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ProcessedArg {
    Plain(String),
    Lookback { index: usize },
    Ellipsis,
}

#[derive(Debug)]
pub(crate) struct ArgVec<'c, 'ns> {
    config: &'c DemangleConfig,
    namespace: Option<&'ns str>,
    args: Vec<ProcessedArg>,

    /// !HACK(c++filt): Allows to avoid emitting an space between a comma and
    /// the ellipsis.
    /// This is will always be `false` if `DemangleConfig::ellipsis_emit_space_after_comma`
    /// is set to `true`. Ellipsis will be handled as yet another element
    /// inside the `args` vector.
    trailing_ellipsis: bool,
}

impl<'c, 'ns> ArgVec<'c, 'ns> {
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
                ProcessedArg::Plain(p) => break Some(p),
                ProcessedArg::Lookback { index: i } => {
                    if *i >= index {
                        break None;
                    }
                    index = *i;
                }
                ProcessedArg::Ellipsis => break Some("..."),
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

        // Map the external `DemangledArg` representation to our `ProcessedArg`
        // internal one.
        let arg = match arg {
            DemangledArg::Plain(plain) => ProcessedArg::Plain(plain),
            DemangledArg::FunctionPointer(function_pointer) => {
                ProcessedArg::Plain(function_pointer.to_string())
            }
            DemangledArg::MethodPointer(method_pointer) => {
                ProcessedArg::Plain(method_pointer.to_string())
            }
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

                for _ in 0..count.get() - 1 {
                    self.args.push(ProcessedArg::Lookback { index });
                }
                ProcessedArg::Lookback { index }
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
                ProcessedArg::Ellipsis
            }
        };
        self.args.push(arg);
        Ok(found_end)
    }

    pub(crate) fn join(self) -> String {
        let mut args = Vec::with_capacity(self.args.len());

        for arg in &self.args {
            match arg {
                ProcessedArg::Plain(plain) => args.push(plain.as_str()),
                ProcessedArg::Lookback { index } => {
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
                    args.push(arg);
                }
                ProcessedArg::Ellipsis => args.push("..."),
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
    template_args: &ArgVec,
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
    template_args: &ArgVec,
    allow_data_after_ellipsis: bool,
) -> Result<(&'s str, ArgVec<'c, 'ns>), DemangleError<'s>> {
    let mut arguments = ArgVec::new(config, namespace);

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
