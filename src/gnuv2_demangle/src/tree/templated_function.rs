/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::string::String;
use core::fmt;

use crate::tree::{ArrayQualifiers, MethodQualifier, NamespaceKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemplatedFunction<'s> {
    func_name: &'s str,
    method_qualifier: MethodQualifier,
    template_args: String,
    argument_list: String,
    return_type: String,
    specialization_namespace: Option<String>,
    array_qualifiers: Option<(ArrayQualifiers, bool)>,
    typ: Option<NamespaceKind<'s>>,
}

impl<'s> TemplatedFunction<'s> {
    #[expect(clippy::too_many_arguments)]
    pub(crate) const fn new(
        func_name: &'s str,
        method_qualifier: MethodQualifier,
        template_args: String,
        argument_list: String,
        return_type: String,
        specialization_namespace: Option<String>,
        array_qualifiers: Option<(ArrayQualifiers, bool)>,
        typ: Option<NamespaceKind<'s>>,
    ) -> Self {
        Self {
            func_name,
            method_qualifier,
            template_args,
            argument_list,
            return_type,
            specialization_namespace,
            array_qualifiers,
            typ,
        }
    }
}

impl fmt::Display for TemplatedFunction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            func_name,
            method_qualifier,
            template_args,
            argument_list,
            return_type,
            specialization_namespace,
            array_qualifiers,
            typ,
        } = self;

        write!(f, "{return_type}")?;
        if let Some(specialization_namespace) = specialization_namespace {
            write!(f, " {specialization_namespace}")?;
        }
        if let Some((array_qualifiers, fix_array_in_return_position)) = array_qualifiers.as_ref() {
            if *fix_array_in_return_position {
                write!(f, " ({}", array_qualifiers.inner_post_qualifiers)?;
            } else {
                write!(f, "{array_qualifiers} ")?;
            }
        } else {
            write!(f, " ")?;
        }
        if let Some(typ) = typ {
            write!(f, "{typ}::")?;
        }

        write!(f, "{func_name}<{}", template_args)?;
        if template_args.ends_with('>') {
            write!(f, " ")?;
        }
        write!(f, ">({argument_list}){method_qualifier}")?;

        if let Some((array_qualifiers, fix_array_in_return_position)) = array_qualifiers.as_ref() {
            if *fix_array_in_return_position {
                write!(f, "){}", array_qualifiers.arrays)?;
            }
        }

        Ok(())
    }
}
