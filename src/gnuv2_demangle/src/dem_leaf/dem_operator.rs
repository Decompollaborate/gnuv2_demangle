/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use super::dem_arg::demangle_argument;

use crate::{
    remainer::Remaining,
    tree::{ArgVec, DemangledArg, Operator},
    DemangleConfig, DemangleError,
};

pub(crate) fn demangle_operator<'s>(
    s: &'s str,
    config: &DemangleConfig,
    allow_array_fixup: bool,
) -> Result<Option<Remaining<'s, Operator>>, DemangleError<'s>> {
    let end_index = s.find("__").ok_or(DemangleError::InvalidSpecialMethod(s))?;
    let op = &s[..end_index];

    // Skip the underscore
    let remaining = &s[end_index + 2..];

    let operator = match op {
        // Memory
        "nw" => Operator::New,
        "dl" => Operator::Delete,
        "vn" => Operator::VectorNew,
        "vd" => Operator::VectorDelete,

        // Comparison
        "eq" => Operator::Equal,
        "ne" => Operator::NotEqual,
        "lt" => Operator::LessThan,
        "gt" => Operator::GreaterThan,
        "le" => Operator::LessEqual,
        "ge" => Operator::GreaterEqual,

        // Assignment
        "as" => Operator::Assignment,
        "apl" => Operator::AssignmentPlus,
        "ami" => Operator::AssignmentMinus,
        "aml" => Operator::AssignmentMultiplication,
        "adv" => Operator::AssignmentDivision,
        "amd" => Operator::AssignmentModulo,
        "aer" => Operator::AssignmentBitwiseExclusiveOr,
        "aad" => Operator::AssignmentBitwiseAnd,
        "aor" => Operator::AssignmentBitwiseOr,
        "als" => Operator::AssignmentLeftShift,
        "ars" => Operator::AssignmentRightShift,

        // Bitwise
        "er" => Operator::BitwiseExclusiveOr,
        "ad" => Operator::BitwiseAnd,
        "or" => Operator::BitwiseOr,
        "ls" => Operator::LeftShift,
        "rs" => Operator::RightShift,
        "co" => Operator::BitwiseNot,

        // Increment/Decrement
        "pp" => Operator::PlusPlus,
        "mm" => Operator::MinusMinus,

        // Logical
        "aa" => Operator::LogicalAnd,
        "oo" => Operator::LogicalOr,
        "nt" => Operator::LogicalNot,

        // Member access
        "vc" => Operator::VectorSubcript,
        "rf" => Operator::Arrow,
        "rm" => Operator::ArrowIndirect,

        // Arithmetic
        "pl" => Operator::Plus,
        "mi" => Operator::Minus,
        "ml" => Operator::Multiplication,
        "dv" => Operator::Division,
        "md" => Operator::Modulo,

        // Other
        "cl" => Operator::Call,
        "cm" => Operator::Comma,

        _ => {
            let Some(cast) = op.strip_prefix("op") else {
                return Ok(None);
            };
            let (remaining, DemangledArg::Plain(typ, array_qualifiers)) = demangle_argument(
                config,
                cast,
                &ArgVec::new(config, None),
                &ArgVec::new(config, None),
                allow_array_fixup,
            )?
            else {
                return Err(DemangleError::UnrecognizedSpecialMethod(op));
            };
            if !remaining.is_empty() {
                return Err(DemangleError::MalformedCastOperatorOverload(remaining));
            }

            Operator::UserDefined {
                typ,
                array_qualifiers,
            }
        }
    };

    let r = Remaining::new(remaining, operator);
    Ok(Some(r))
}
