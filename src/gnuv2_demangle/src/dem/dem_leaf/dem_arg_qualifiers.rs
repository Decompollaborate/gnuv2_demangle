/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;

use crate::{
    remainer::{Remaining, StrParsing},
    tree::leaf::{
        ArgPostQualifier, ArgPostQualifierVec, ArgQualifiers, ArrayQualifiers, ArrayVec, Signedness,
    },
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_arg_qualifiers<'s>(
    config: &DemangleConfig,
    s: &'s str,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, ArgQualifiers>, DemangleError<'s>> {
    let Remaining {
        r,
        d: (sign, post_qualifiers),
    } = demangle_arg_post_qualifiers(s)?;
    let Remaining { r, d: qualifiers } =
        demangle_array_pseudo_qualifier(config, r, sign, post_qualifiers, allow_array_fixup)?;

    Ok(Remaining::new(r, qualifiers))
}

fn demangle_arg_post_qualifiers<'s>(
    s: &'s str,
) -> Result<Remaining<'s, (Signedness, ArgPostQualifierVec)>, DemangleError<'s>> {
    let mut remaining = s;
    let mut post_qualifiers = ArgPostQualifierVec::new();

    while !remaining.is_empty() {
        let Remaining { r, d: c } = remaining
            .p_first()
            .ok_or(DemangleError::RanOutOfArguments)?;

        let post = match c {
            'P' => ArgPostQualifier::Pointer,
            'R' => ArgPostQualifier::Reference,
            'C' => ArgPostQualifier::Const,
            'V' => ArgPostQualifier::Volatile,
            _ => break,
        };

        post_qualifiers.push(post);
        remaining = r;
    }

    // There can be at most one signedness qualifier as far as I know
    let (remaining, sign) = if let Some(Remaining { r, d: c }) = remaining.p_first() {
        match c {
            'S' => (r, Signedness::Signed),
            'U' => (r, Signedness::Unsigned),
            _ => (remaining, Signedness::No),
        }
    } else {
        (remaining, Signedness::No)
    };

    Ok(Remaining::new(remaining, (sign, post_qualifiers)))
}

// `allow_array_fixup` exists because array sizes are not always messed up.
// As far as I know, array sizes are correct only on templated functions.
fn demangle_array_pseudo_qualifier<'s>(
    config: &DemangleConfig,
    s: &'s str,
    sign: Signedness,
    post_qualifiers: ArgPostQualifierVec,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, ArgQualifiers>, DemangleError<'s>> {
    if !s.starts_with('A') {
        let qualifiers = ArgQualifiers {
            sign,
            post_qualifiers,
            array_qualifiers: None.into(),
        };
        return Ok(Remaining::new(s, qualifiers));
    }

    if sign != Signedness::No {
        // Avoid stuff like "signed signed"
        return Err(DemangleError::PrevQualifiersInInvalidPostioniAtArrayArgument(s));
    }

    let mut arrays = Vec::new();

    let mut args = s;
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

        let array_length = if config.fix_array_length_arg && allow_array_fixup {
            array_length + 1
        } else {
            array_length
        };

        arrays.push(array_length);
        args = remaining;
    }

    let array_qualifiers = ArrayQualifiers::new(post_qualifiers, ArrayVec::new(arrays));

    let Remaining {
        r,
        d: (sign_other, post),
    } = demangle_arg_post_qualifiers(args)?;

    let qualifiers = ArgQualifiers::new(sign_other, post, Some(array_qualifiers).into());
    Ok(Remaining::new(r, qualifiers))
}
