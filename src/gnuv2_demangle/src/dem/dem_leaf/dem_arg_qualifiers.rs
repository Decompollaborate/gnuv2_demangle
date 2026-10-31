/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::vec::Vec;

use crate::{
    remainer::{Remaining, StrParsing},
    tree::leaf::{
        ArgPostQualifier, ArgPostQualifierVec, ArgPreQualifier, ArgPreQualifierVec, ArgQualifiers,
        ArrayQualifiers, ArrayVec, Signedness,
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
        d: (sign, post_qualifiers, pre_qualifiers),
    } = demangle_arg_post_qualifiers(s)?;
    let Remaining { r, d: qualifiers } = demangle_array_pseudo_qualifier(
        config,
        r,
        sign,
        post_qualifiers,
        pre_qualifiers,
        allow_array_fixup,
    )?;

    Ok(Remaining::new(r, qualifiers))
}

fn demangle_arg_post_qualifiers<'s>(
    s: &'s str,
) -> Result<Remaining<'s, (Signedness, ArgPostQualifierVec, ArgPreQualifierVec)>, DemangleError<'s>>
{
    let Remaining {
        r: remaining,
        d: post_qualifiers,
    } = demangle_post_qualifiers(s)?;

    // There can be at most one signedness qualifier as far as I know
    let (remaining, sign) = remaining
        .p_first()
        .and_then(|Remaining { r, d: c }| Signedness::from_char(c).map(|s| (r, s)))
        .unwrap_or((remaining, Signedness::No));

    let Remaining {
        r,
        d: pre_qualifiers,
    } = demangle_pre_qualifiers(remaining)?;

    Ok(Remaining::new(r, (sign, post_qualifiers, pre_qualifiers)))
}

// `allow_array_fixup` exists because array sizes are not always messed up.
// As far as I know, array sizes are correct only on templated functions.
fn demangle_array_pseudo_qualifier<'s>(
    config: &DemangleConfig,
    s: &'s str,
    sign: Signedness,
    post_qualifiers: ArgPostQualifierVec,
    pre_qualifiers: ArgPreQualifierVec,
    allow_array_fixup: bool,
) -> Result<Remaining<'s, ArgQualifiers>, DemangleError<'s>> {
    if !s.starts_with('A') {
        let qualifiers = ArgQualifiers::new(sign, post_qualifiers, pre_qualifiers, None);
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

    let array_qualifiers =
        ArrayQualifiers::new(post_qualifiers, pre_qualifiers, ArrayVec::new(arrays));

    let Remaining {
        r,
        d: (sign_other, post, pre),
    } = demangle_arg_post_qualifiers(args)?;

    let qualifiers = ArgQualifiers::new(sign_other, post, pre, Some(array_qualifiers));
    Ok(Remaining::new(r, qualifiers))
}

fn demangle_post_qualifiers<'s>(
    s: &'s str,
) -> Result<Remaining<'s, ArgPostQualifierVec>, DemangleError<'s>> {
    let mut remaining = s;
    let mut post_qualifiers = ArgPostQualifierVec::new();

    while !remaining.is_empty() {
        let Remaining { r, d: c } = remaining
            .p_first()
            .ok_or(DemangleError::RanOutOfArguments)?;

        let Some(post) = ArgPostQualifier::from_char(c) else {
            break;
        };

        post_qualifiers.push(post);
        remaining = r;
    }

    let out = Remaining::new(remaining, post_qualifiers);
    Ok(out)
}

fn demangle_pre_qualifiers<'s>(
    s: &'s str,
) -> Result<Remaining<'s, ArgPreQualifierVec>, DemangleError<'s>> {
    let mut remaining = s;
    let mut pre_qualifiers = ArgPreQualifierVec::new();

    while let Some(Remaining { r, d: c }) = remaining.p_first() {
        let Some(pre) = ArgPreQualifier::from_char(c) else {
            break;
        };

        if !pre_qualifiers.push(pre) {
            return Err(DemangleError::InvalidPreQualifier(remaining));
        }
        remaining = r;
    }

    let out = Remaining::new(remaining, pre_qualifiers);
    Ok(out)
}
