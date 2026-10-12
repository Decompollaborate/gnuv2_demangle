/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, string::String};
use core::fmt;

use crate::{
    dem::demangle_custom_name,
    dem_arg_list::ArgVec,
    dem_namespace::{demangle_namespaces, NamespaceVec},
    dem_template::demangle_template,
    remainer::Remaining,
    DemangleConfig, DemangleError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NamespacedType<'s> {
    pub(crate) namespace: NamespaceKind<'s>,
    pub(crate) type_name: &'s str,
}

impl<'s> NamespacedType<'s> {
    const fn new(namespace: NamespaceKind<'s>, type_name: &'s str) -> Self {
        Self {
            namespace,
            type_name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum NamespaceKind<'s> {
    Template(String),
    Namespace(NamespaceVec<'s>),
    Class(&'s str),
}

impl<'s> fmt::Display for NamespaceKind<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Template(t) => write!(f, "{t}"),
            Self::Namespace(ns) => write!(f, "{ns}"),
            Self::Class(c) => write!(f, "{c}"),
        }
    }
}

// silly
impl<'s> From<&NamespaceKind<'s>> for Cow<'s, str> {
    fn from(value: &NamespaceKind<'s>) -> Self {
        match value {
            NamespaceKind::Template(template) => Cow::from(template.clone()),
            NamespaceKind::Namespace(namespace_vec) => Cow::from(namespace_vec),
            NamespaceKind::Class(class) => Cow::from(*class),
        }
    }
}

pub(crate) fn demangle_namespaced_type<'s, F>(
    config: &DemangleConfig,
    s: &'s str,
    allow_array_fixup: bool,
    class_name_err: F,
) -> Result<Option<Remaining<'s, NamespacedType<'s>>>, DemangleError<'s>>
where
    F: Fn(&'s str) -> DemangleError<'s>,
{
    let nt = if s.starts_with(|c| matches!(c, '1'..='9')) {
        let Remaining { r, d: class_name } = demangle_custom_name(s, class_name_err)?;

        let nt = NamespacedType::new(NamespaceKind::Class(class_name), class_name);

        Some(Remaining::new(r, nt))
    } else if let Some(s) = s.strip_prefix('t') {
        let (r, template, typ) =
            demangle_template(config, s, &ArgVec::new(config, None), allow_array_fixup)?;

        let nt = NamespacedType::new(NamespaceKind::Template(template), typ);

        Some(Remaining::new(r, nt))
    } else if let Some(s) = s.strip_prefix('Q') {
        let ns = demangle_namespaces(config, s, &ArgVec::new(config, None), allow_array_fixup)?
            .convert_data(|(namespaces, trailing_namespace)| {
                NamespacedType::new(NamespaceKind::Namespace(namespaces), trailing_namespace)
            });

        Some(ns)
    } else {
        None
    };

    Ok(nt)
}
