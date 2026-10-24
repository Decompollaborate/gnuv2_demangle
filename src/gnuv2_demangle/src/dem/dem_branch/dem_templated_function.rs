/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::sync::Arc;

use crate::{
    dem::dem_leaf::{
        demangle_argument, demangle_argument_list_impl, demangle_method_qualifier,
        demangle_namespaced_type, demangle_template_with_return_type,
    },
    remainer::Remaining,
    tree::{
        branch::TemplatedFunction,
        leaf::{ArgVec, DemangledArg, PlainArgArray},
    },
    DemangleConfig, DemangleError,
};

/// Templated functions and methods.
///
/// A templated method is templated individually, it doesn't matter if the
/// class it comes from is templated or not.
pub(crate) fn demangle_templated_function<'s>(
    config: &DemangleConfig,
    func_name: &'s str,
    s: &'s str,
) -> Result<TemplatedFunction<'s>, DemangleError<'s>> {
    // Arrays do need to be fixed up if it appears in the template list, but
    // not in the rest of the definition.
    let allow_array_fixup = true;
    let (remaining, template_args, typ) =
        demangle_template_with_return_type(config, s, allow_array_fixup)?;
    let allow_array_fixup = false;

    let Remaining {
        r: remaining,
        d: method_qualifier,
    } = demangle_method_qualifier(remaining);

    let (remaining, typ) = if let Some(typ) = typ {
        (remaining, Some(typ))
    } else if let Some(nt) = demangle_namespaced_type(
        config,
        remaining,
        allow_array_fixup,
        DemangleError::InvalidNamespaceOnTemplatedFunction,
    )? {
        let Remaining { r, d: nt } = nt;
        (r, Some(nt))
    } else {
        (remaining, None)
    };
    let typ = typ.map(Arc::new);

    // - First there's an optional namespace used in template specializations.
    // - Then there's the optional argument list, which may be omitted if there
    //   are no parameters.
    // - Finally the return type.

    // Demangle the specialization namespace
    let (remaining, specialization_namespace) = if let Some(r) = remaining.strip_prefix('_') {
        let (r, DemangledArg::Plain(specialization_namespace)) = demangle_argument(
            config,
            r,
            &ArgVec::new(config, typ.as_ref().map(Arc::clone)),
            &template_args,
            allow_array_fixup,
        )?
        else {
            return Err(DemangleError::MalformedTemplatedSpecializationInvalidNamespace(r));
        };

        (r, Some(specialization_namespace))
    } else {
        (remaining, None)
    };

    // Demangle the argument list
    let Remaining {
        r: remaining,
        d: argument_list,
    } = demangle_argument_list_impl(
        config,
        remaining,
        typ.as_ref().map(Arc::clone),
        &template_args,
        false,
        allow_array_fixup,
    )?;

    // Demangle the return type
    let (specialization_namespace, return_type) = if let Some(r) = remaining.strip_prefix('_') {
        let (r, DemangledArg::Plain(ret_type)) = demangle_argument(
            config,
            r,
            &ArgVec::new(config, typ.as_ref().map(Arc::clone)),
            &template_args,
            allow_array_fixup,
        )?
        else {
            return Err(DemangleError::MalformedTemplateWithReturnTypeMissingReturnType(r));
        };

        if !r.is_empty() {
            return Err(
                DemangleError::TrailingDataAfterReturnTypeOfMalformedTemplateWithReturnType(r),
            );
        }

        (specialization_namespace, ret_type)
    } else if let Some(actual_return_type) = specialization_namespace {
        // If there's no argument list and this symbol is not a template
        // specialization inside a namespace then we mistakenly consumed the
        // return type as the specialization_namespace

        if !remaining.is_empty() {
            return Err(
                DemangleError::TrailingDataAfterReturnTypeOfTemplatedSpecialization(remaining),
            );
        }

        (None, actual_return_type)
    } else {
        return Err(DemangleError::MalformedTemplateWithReturnTypeMissingReturnType(remaining));
    };

    let PlainArgArray {
        plain: return_type,
        array_qualifiers,
    } = return_type;

    let array_qualifiers = array_qualifiers.map(|x| (x, config.fix_array_in_return_position));

    Ok(TemplatedFunction::new(
        func_name,
        method_qualifier,
        template_args,
        argument_list,
        return_type,
        specialization_namespace.map(|x| x.plain),
        array_qualifiers,
        typ,
    ))
}
