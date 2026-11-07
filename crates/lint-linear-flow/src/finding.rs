use std::fmt;

use syn::{BinOp, Expr, UnOp};

use lint_core::Fix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    LinearBodies,
    PureRouters,
    PlainConditions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finding {
    If,
    IfLet,
    Match,
    While,
    For,
    Loop,
    LetElse,
    Guard,
    ValuePattern,
    ArmLogic,
    And,
    Or,
    Not,
}

impl Finding {
    const COMBINATORS: &'static str = "drive an iterator with combinators (map, filter, fold, try_fold, take_while)";
    const CONDITION: &'static str = "name the condition as an enum or a predicate type";

    pub const fn branching(condition: &Expr) -> Self {
        match condition {
            Expr::Let(_) => Self::IfLet,
            _ => Self::If,
        }
    }

    pub const fn joining(operator: BinOp) -> Option<Self> {
        match operator {
            BinOp::And(_) => Some(Self::And),
            BinOp::Or(_) => Some(Self::Or),
            _ => None,
        }
    }

    pub const fn negating(operator: UnOp) -> Option<Self> {
        match operator {
            UnOp::Not(_) => Some(Self::Not),
            _ => None,
        }
    }

    pub const fn rule(self) -> Rule {
        match self {
            Self::If | Self::IfLet | Self::Match | Self::While | Self::For | Self::Loop | Self::LetElse => {
                Rule::LinearBodies
            }
            Self::Guard | Self::ValuePattern | Self::ArmLogic => Rule::PureRouters,
            Self::And | Self::Or | Self::Not => Rule::PlainConditions,
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::If => formatter.write_str("if"),
            Self::IfLet => formatter.write_str("if let that is not the whole body"),
            Self::Match => formatter.write_str("match that is not the whole body"),
            Self::While => formatter.write_str("while loop"),
            Self::For => formatter.write_str("for loop"),
            Self::Loop => formatter.write_str("loop"),
            Self::LetElse => formatter.write_str("let-else"),
            Self::Guard => formatter.write_str("match guard in a router"),
            Self::ValuePattern => formatter.write_str("router arm matches a literal, range, tuple or slice"),
            Self::ArmLogic => formatter.write_str("router arm is not a single call or value"),
            Self::And => formatter.write_str("boolean operator &&"),
            Self::Or => formatter.write_str("boolean operator ||"),
            Self::Not => formatter.write_str("boolean operator !"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        match self {
            Self::If => Fix::new(
                "use combinators on Option, Result and iterators (then_some, filter, map_or), or make the whole body one match on an enum, Option or Result",
            ),
            Self::IfLet => Fix::new(
                "use combinators on Option and Result (map, and_then, map_or), or extract it into its own function whose whole body is the if let",
            ),
            Self::Match => Fix::new(
                "extract it into its own function whose whole body is the match, or use combinators on Option, Result and iterators",
            ),
            Self::While | Self::For => Fix::new(Self::COMBINATORS),
            Self::Loop => {
                Fix::new("drive an iterator with combinators (successors, repeat_with, take_while, try_fold)")
            }
            Self::LetElse => Fix::new("use combinators on Option and Result, or ? after ok_or"),
            Self::Guard => Fix::new("name the condition as an enum or a predicate type and match on it"),
            Self::ValuePattern => Fix::new("match on an enum, Option or Result that names the cases"),
            Self::ArmLogic => Fix::new("extract the arm's logic into its own linear function and call it"),
            Self::And | Self::Or | Self::Not => Fix::new(Self::CONDITION),
        }
    }
}
