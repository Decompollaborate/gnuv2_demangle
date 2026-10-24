/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::{borrow::Cow, string::ToString};
use core::num::NonZeroU32;

use super::{demangle_custom_name, demangle_template};

use crate::{
    demangle,
    remainer::{Remaining, StrParsing},
    tree::leaf::{CustomName, NamespaceKind, NamespaceVec, NamespacedType, TemplatedValueVec},
    DemangleConfig, DemangleError,
};

// 'Q' must be stripped already
pub(crate) fn demangle_namespaces<'s>(
    config: &DemangleConfig,
    s: &'s str,
    template_args: &TemplatedValueVec<'s>,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, NamespacedType<'s>>, DemangleError<'s>> {
    let Remaining {
        r,
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
        NonZeroU32::new(namespace_count).ok_or(DemangleError::InvalidNamespaceCount(s))?;

    demangle_namespaces_impl(config, r, namespace_count, template_args, allow_array_fixup)
}

fn demangle_namespaces_impl<'s>(
    config: &DemangleConfig,
    s: &'s str,
    namespace_count: NonZeroU32,
    template_args: &TemplatedValueVec<'s>,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, NamespacedType<'s>>, DemangleError<'s>> {
    let mut namespaces = NamespaceVec::new();
    let mut remaining = s;
    let mut trailing_type = CustomName::new("");

    for _i in 0..namespace_count.get() {
        // Sometimes there's a trailing underscore after a number.
        // Not sure if this is the correct way to handle this, but at least it
        // doesn't seem to break anything else.
        // i.e. CreateRoadBlock__12AICopManagerP8IPursuitiP8IVehiclePQ43UTL11Collectionst11ListableSet4Z8IVehiclei10Z12eVehicleListUi10_4List
        remaining = remaining.trim_start_matches('_');

        let (r, n) = if let Some(temp) = remaining.strip_prefix('t') {
            let Remaining { r, d: t } =
                demangle_template(config, temp, template_args, allow_array_fixup)?;
            trailing_type = t.class_name();
            (r, Cow::from(t.to_string()))
        } else {
            let Remaining { r, d: ns } =
                demangle_custom_name(remaining, DemangleError::InvalidCustomNameOnNamespace)?;
            trailing_type = ns;

            // Since the current symbol may actually be defined inside a local
            // struct inside an outer function (an static function from said
            // struct or a class destructor, etc), then this namespace may
            // actually need further demangling.
            // If we fail, then we just keep the old namespace.
            let new_ns = match demangle(ns.into_raw(), config) {
                Ok(new_ns) => Cow::from(new_ns.to_string()),
                Err(_e) => Cow::from(ns),
            };

            (r, new_ns)
        };
        remaining = r;
        namespaces.push(n);
    }

    let namespaces = NamespaceKind::Namespace(namespaces);
    let nst = NamespacedType::new(namespaces, trailing_type);
    Ok(Remaining::new(remaining, nst))
}
