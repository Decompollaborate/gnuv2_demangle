/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::fmt;

use super::{DemangledArg, NamespacedType};

use crate::{DemangleConfig, DemangleError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgVec<'c, 'ns> {
    config: &'c DemangleConfig,
    // Use Arc to avoid cloning the namespace around
    namespace: Option<Arc<NamespacedType<'ns>>>,
    args: Vec<ProcessedArg>,

    /// !HACK(c++filt): Allows to avoid emitting an space between a comma and
    /// the ellipsis.
    /// This is will always be `false` if `DemangleConfig::ellipsis_emit_space_after_comma`
    /// is set to `true`. Ellipsis will be handled as yet another element
    /// inside the `args` vector.
    trailing_ellipsis: bool,
}

impl<'c, 'ns> ArgVec<'c, 'ns> {
    pub(crate) fn new(
        config: &'c DemangleConfig,
        namespace: Option<Arc<NamespacedType<'ns>>>,
    ) -> Self {
        Self {
            config,
            namespace,
            args: Vec::new(),
            trailing_ellipsis: false,
        }
    }

    pub(crate) fn len(&self) -> usize {
        let extra = if self.namespace.is_some() { 1 } else { 0 };
        self.args.len() + extra
    }

    pub(crate) fn get<'p>(&'p self, mut index: usize) -> Option<ResolvedProcessedArg<'ns, 'p>> {
        if let Some(namespace) = &self.namespace {
            if index == 0 {
                return Some(ResolvedProcessedArg::Namespace(Arc::clone(namespace)));
            } else {
                index -= 1;
            }
        }

        loop {
            let arg = self.args.get(index)?;
            match arg {
                ProcessedArg::Plain(p) => break Some(ResolvedProcessedArg::Plain(p)),
                ProcessedArg::Lookback { index: i } => {
                    if *i >= index {
                        break None;
                    }
                    index = *i;
                }
                ProcessedArg::Ellipsis => break Some(ResolvedProcessedArg::Ellipsis),
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
            DemangledArg::Plain(plain, array_qualifiers) => {
                ProcessedArg::Plain(format!("{plain}{array_qualifiers}"))
            }
            DemangledArg::FunctionPointer(function_pointer) => {
                ProcessedArg::Plain(function_pointer.to_string())
            }
            DemangledArg::MethodPointer(method_pointer) => {
                ProcessedArg::Plain(method_pointer.to_string())
            }
            DemangledArg::ObjectPointer(object) => ProcessedArg::Plain(object),
            DemangledArg::Repeat { count, index } => {
                // Check the index is in-bounds
                if index >= self.len() {
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
                ProcessedArg::Plain(plain) => args.push(Cow::from(plain.as_str())),
                ProcessedArg::Lookback { index } => {
                    let arg = if let Some(namespace) = &self.namespace {
                        if *index == 0 {
                            Cow::from(&namespace.namespace)
                        } else {
                            Cow::clone(
                                args.get(*index - 1)
                                    .expect("Indices were verified when pushing the arguments"),
                            )
                        }
                    } else {
                        Cow::clone(
                            args.get(*index)
                                .expect("Indices were verified when pushing the arguments"),
                        )
                    };
                    args.push(arg);
                }
                ProcessedArg::Ellipsis => args.push(Cow::from("...")),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ProcessedArg {
    Plain(String),
    Lookback { index: usize },
    Ellipsis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ResolvedProcessedArg<'ns, 'p> {
    Namespace(Arc<NamespacedType<'ns>>),
    Plain(&'p str),
    Ellipsis,
}

impl fmt::Display for ResolvedProcessedArg<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Namespace(ns) => write!(f, "{}", ns.namespace),
            Self::Plain(p) => write!(f, "{p}"),
            Self::Ellipsis => write!(f, "..."),
        }
    }
}

// silly
impl<'s> From<ResolvedProcessedArg<'s, 's>> for Cow<'s, str> {
    fn from(value: ResolvedProcessedArg<'s, 's>) -> Self {
        match value {
            ResolvedProcessedArg::Namespace(ntn) => Cow::from(&ntn.namespace),
            ResolvedProcessedArg::Plain(p) => Cow::from(p),
            ResolvedProcessedArg::Ellipsis => Cow::from("..."),
        }
    }
}
