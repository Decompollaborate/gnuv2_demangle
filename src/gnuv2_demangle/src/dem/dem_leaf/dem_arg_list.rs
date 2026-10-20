/* SPDX-FileCopyrightText: © 2025 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::sync::Arc;

use super::{demangle_argument, demangle_static_suffix};

use crate::{
    remainer::Remaining,
    tree::leaf::{ArgVec, NamespacedType, StaticSuffix},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_argument_list<'s, 'ns>(
    config: &DemangleConfig,
    args: &'s str,
    namespace: Option<Arc<NamespacedType<'ns>>>,
    template_args: &ArgVec,
    allow_array_fixup: bool,
) -> Result<(ArgVec<'ns>, StaticSuffix<'s>), DemangleError<'s>> {
    let Remaining {
        r: remaining,
        d: argument_list,
    } = demangle_argument_list_impl(
        config,
        args,
        namespace,
        template_args,
        false,
        allow_array_fixup,
    )?;

    // TODO: Consider moving this call to demangle_argument_list_impl
    let Remaining {
        r: remaining,
        d: static_suffix,
    } = demangle_static_suffix(config, remaining)?;

    if !remaining.is_empty() {
        return Err(DemangleError::TrailingDataAfterArgumentList(remaining));
    }

    Ok((argument_list, static_suffix))
}

pub(crate) fn demangle_argument_list_impl<'s, 'ns>(
    config: &DemangleConfig,
    mut args: &'s str,
    namespace: Option<Arc<NamespacedType<'ns>>>,
    template_args: &ArgVec,
    allow_data_after_ellipsis: bool,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, ArgVec<'ns>>, DemangleError<'s>> {
    let mut arguments = ArgVec::new(config, namespace);

    while !args.is_empty() && !args.starts_with('_') {
        if config.demangle_static_suffix && args.starts_with('.') {
            // g++ emits a dot and numeric suffix to handle statics symbols and
            // avoid them from overlapping between them
            // c++filt chokes on them and refuses to demangle them.
            break;
        }

        let old_args = args;
        let (remaining, b) = demangle_argument(
            config,
            old_args,
            &arguments,
            template_args,
            allow_array_fixup,
        )?;

        args = remaining;
        if arguments.push(b, old_args, remaining, allow_data_after_ellipsis)? {
            break;
        }
    }

    Ok(Remaining::new(args, arguments))
}
