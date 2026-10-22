/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::fmt;

use super::{DemangledArg, FunctionPointer, MethodPointer, NamespacedType, PlainArgArray};

use crate::{DemangleConfig, DemangleError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArgVec<'ns> {
    // Use Arc to avoid cloning the namespace around
    namespace: Option<Arc<NamespacedType<'ns>>>,
    args: Vec<ProcessedArg<'ns>>,

    ellipsis_emit_space_after_comma: bool,

    /// !HACK(c++filt): Allows to avoid emitting an space between a comma and
    /// the ellipsis.
    /// This is will always be `false` if `DemangleConfig::ellipsis_emit_space_after_comma`
    /// is set to `true`. Ellipsis will be handled as yet another element
    /// inside the `args` vector.
    trailing_ellipsis: bool,
}

impl<'ns> ArgVec<'ns> {
    pub(crate) fn new(
        config: &DemangleConfig,
        namespace: Option<Arc<NamespacedType<'ns>>>,
    ) -> Self {
        Self {
            namespace,
            args: Vec::new(),
            ellipsis_emit_space_after_comma: config.ellipsis_emit_space_after_comma,
            trailing_ellipsis: false,
        }
    }

    pub(crate) fn len(&self) -> usize {
        let extra = if self.namespace.is_some() { 1 } else { 0 };
        self.args.len() + extra
    }

    pub(crate) fn get(&self, mut index: usize) -> Option<ResolvedProcessedArg<'ns>> {
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
                ProcessedArg::Plain(p) => break Some(ResolvedProcessedArg::Plain(Arc::clone(p))),
                ProcessedArg::FunctionPointer(p) => {
                    break Some(ResolvedProcessedArg::FunctionPointer(Arc::clone(p)))
                }
                ProcessedArg::MethodPointer(p) => {
                    break Some(ResolvedProcessedArg::MethodPointer(Arc::clone(p)))
                }
                ProcessedArg::ObjectPointer(p) => {
                    break Some(ResolvedProcessedArg::ObjectPointer(Arc::clone(p)))
                }
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
        arg: DemangledArg<'ns>,
        s: &'s str,
        remaining: &'s str,
        allow_data_after_ellipsis: bool,
    ) -> Result<bool, DemangleError<'s>> {
        let mut found_end = false;

        // Map the external `DemangledArg` representation to our `ProcessedArg`
        // internal one.
        let arg = match arg {
            DemangledArg::Plain(plain) => ProcessedArg::Plain(Arc::new(plain)),
            DemangledArg::FunctionPointer(function_pointer) => {
                ProcessedArg::FunctionPointer(Arc::new(function_pointer))
            }
            DemangledArg::MethodPointer(method_pointer) => {
                ProcessedArg::MethodPointer(Arc::new(method_pointer))
            }
            DemangledArg::ObjectPointer(object) => ProcessedArg::ObjectPointer(Arc::new(object)),
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
                if !self.ellipsis_emit_space_after_comma {
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
                ProcessedArg::Plain(plain) => args.push(Cow::from(plain.to_string())),
                ProcessedArg::FunctionPointer(plain) => args.push(Cow::from(plain.to_string())),
                ProcessedArg::MethodPointer(plain) => args.push(Cow::from(plain.to_string())),
                ProcessedArg::ObjectPointer(plain) => args.push(Cow::from(plain.as_str())),
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

impl fmt::Display for ArgVec<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.trailing_ellipsis && self.args.is_empty() {
            write!(f, "void")
        } else {
            // TODO: get rid of clone
            write!(f, "{}", self.clone().join())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ProcessedArg<'ns> {
    Plain(Arc<PlainArgArray<'ns>>),
    FunctionPointer(Arc<FunctionPointer>),
    MethodPointer(Arc<MethodPointer>),
    ObjectPointer(Arc<String>),
    Lookback { index: usize },
    Ellipsis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ResolvedProcessedArg<'ns> {
    Namespace(Arc<NamespacedType<'ns>>),
    Plain(Arc<PlainArgArray<'ns>>),
    FunctionPointer(Arc<FunctionPointer>),
    MethodPointer(Arc<MethodPointer>),
    ObjectPointer(Arc<String>),
    Ellipsis,
}

impl fmt::Display for ResolvedProcessedArg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Namespace(ns) => write!(f, "{}", ns.namespace),
            Self::Plain(p) => write!(f, "{p}"),
            Self::FunctionPointer(p) => write!(f, "{p}"),
            Self::MethodPointer(p) => write!(f, "{p}"),
            Self::ObjectPointer(p) => write!(f, "{p}"),
            Self::Ellipsis => write!(f, "..."),
        }
    }
}

// silly
impl<'s> From<ResolvedProcessedArg<'s>> for Cow<'s, str> {
    fn from(value: ResolvedProcessedArg<'s>) -> Self {
        match value {
            ResolvedProcessedArg::Namespace(ntn) => Cow::from(&ntn.namespace),
            ResolvedProcessedArg::Plain(p) => Cow::from(p.to_string()),
            ResolvedProcessedArg::FunctionPointer(p) => Cow::from(p.to_string()),
            ResolvedProcessedArg::MethodPointer(p) => Cow::from(p.to_string()),
            ResolvedProcessedArg::ObjectPointer(p) => Cow::from(p.to_string()),
            ResolvedProcessedArg::Ellipsis => Cow::from("..."),
        }
    }
}
