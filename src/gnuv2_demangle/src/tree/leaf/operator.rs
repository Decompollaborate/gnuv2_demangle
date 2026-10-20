/* SPDX-FileCopyrightText: © 2026 Decompollaborate */
/* SPDX-License-Identifier: MIT OR Apache-2.0 */

use alloc::string::String;
use core::fmt;

use super::ArrayQualifiers;

use crate::option_display::OptionDisplay;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Operator {
    // Memory
    New,          // "nw"
    Delete,       // "dl"
    VectorNew,    // "vn"
    VectorDelete, // "vd"

    // Comparison
    Equal,        // "eq"
    NotEqual,     // "ne"
    LessThan,     // "lt"
    GreaterThan,  // "gt"
    LessEqual,    // "le"
    GreaterEqual, // "ge"

    // Assignment
    Assignment,                   // "as"
    AssignmentPlus,               // "apl"
    AssignmentMinus,              // "ami"
    AssignmentMultiplication,     // "aml"
    AssignmentDivision,           // "adv"
    AssignmentModulo,             // "amd"
    AssignmentBitwiseExclusiveOr, // "aer"
    AssignmentBitwiseAnd,         // "aad"
    AssignmentBitwiseOr,          // "aor"
    AssignmentLeftShift,          // "als"
    AssignmentRightShift,         // "ars"

    // Bitwise
    BitwiseExclusiveOr, // "er"
    BitwiseAnd,         // "ad"
    BitwiseOr,          // "or"
    LeftShift,          // "ls"
    RightShift,         // "rs"
    BitwiseNot,         // "co"

    // Increment/Decrement
    PlusPlus,   // "pp"
    MinusMinus, // "mm"

    // Logical
    LogicalAnd, // "aa"
    LogicalOr,  // "oo"
    LogicalNot, // "nt"

    // Member access
    VectorSubcript, // "vc"
    Arrow,          // "rf"
    ArrowIndirect,  // "rm"

    // Arithmetic
    Plus,           // "pl"
    Minus,          // "mi"
    Multiplication, // "ml"
    Division,       // "dv"
    Modulo,         // "md"

    // Other
    Call,  // "cl"
    Comma, // "cm"

    UserDefined {
        typ: String,
        array_qualifiers: OptionDisplay<ArrayQualifiers>,
    },
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New => write!(f, "operator new"),
            Self::Delete => write!(f, "operator delete"),
            Self::VectorNew => write!(f, "operator new []"),
            Self::VectorDelete => write!(f, "operator delete []"),

            Self::Equal => write!(f, "operator=="),
            Self::NotEqual => write!(f, "operator!="),
            Self::LessThan => write!(f, "operator<"),
            Self::GreaterThan => write!(f, "operator>"),
            Self::LessEqual => write!(f, "operator<="),
            Self::GreaterEqual => write!(f, "operator>="),

            Self::Assignment => write!(f, "operator="),
            Self::AssignmentPlus => write!(f, "operator+="),
            Self::AssignmentMinus => write!(f, "operator-="),
            Self::AssignmentMultiplication => write!(f, "operator*="),
            Self::AssignmentDivision => write!(f, "operator/="),
            Self::AssignmentModulo => write!(f, "operator%="),
            Self::AssignmentBitwiseExclusiveOr => write!(f, "operator^="),
            Self::AssignmentBitwiseAnd => write!(f, "operator&="),
            Self::AssignmentBitwiseOr => write!(f, "operator|="),
            Self::AssignmentLeftShift => write!(f, "operator<<="),
            Self::AssignmentRightShift => write!(f, "operator>>="),

            Self::BitwiseExclusiveOr => write!(f, "operator^"),
            Self::BitwiseAnd => write!(f, "operator&"),
            Self::BitwiseOr => write!(f, "operator|"),
            Self::LeftShift => write!(f, "operator<<"),
            Self::RightShift => write!(f, "operator>>"),
            Self::BitwiseNot => write!(f, "operator~"),

            Self::PlusPlus => write!(f, "operator++"),
            Self::MinusMinus => write!(f, "operator--"),

            Self::LogicalAnd => write!(f, "operator&&"),
            Self::LogicalOr => write!(f, "operator||"),
            Self::LogicalNot => write!(f, "operator!"),

            Self::VectorSubcript => write!(f, "operator[]"),
            Self::Arrow => write!(f, "operator->"),
            Self::ArrowIndirect => write!(f, "operator->*"),

            Self::Plus => write!(f, "operator+"),
            Self::Minus => write!(f, "operator-"),
            Self::Multiplication => write!(f, "operator*"),
            Self::Division => write!(f, "operator/"),
            Self::Modulo => write!(f, "operator%"),

            Self::Call => write!(f, "operator()"),
            Self::Comma => write!(f, "operator, "),

            Self::UserDefined {
                typ,
                array_qualifiers,
            } => write!(f, "operator {typ}{array_qualifiers}"),
        }
    }
}
