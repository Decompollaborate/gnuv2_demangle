/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use crate::{
    dem::demangle_custom_name,
    dem_namespace::demangle_namespaces,
    dem_template::demangle_template,
    remainer::Remaining,
    tree::{ArgVec, NamespaceKind, NamespacedType},
    DemangleConfig, DemangleError,
};

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
        let t = demangle_template(config, s, &ArgVec::new(config, None), allow_array_fixup)?
            .convert_data(|d| d.into_namespaced_type());

        Some(t)
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
