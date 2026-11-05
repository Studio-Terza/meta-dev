use std::fmt;

use syn::{BinOp, Expr, UnOp};

use lint_core::Fix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    LinearBodies,
    PureRouters,
    PlainConditions,
    IndependentSelections,
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
    IgnoredElement,
}

impl Rule {
    pub const SHAPING: [Rule; 3] = [Rule::LinearBodies, Rule::PureRouters, Rule::PlainConditions];
}

impl Finding {
    const COMBINATORS: &'static str = "drive an iterator with combinators (map, fold, try_fold, take_while, or filter on a predicate that reads its element)";
    const CONDITION: &'static str = "name the condition as an enum or a predicate type";

    pub fn branching(condition: &Expr) -> Finding {
        match condition {
            Expr::Let(_) => Finding::IfLet,
            _ => Finding::If,
        }
    }

    pub fn joining(operator: BinOp) -> Option<Finding> {
        match operator {
            BinOp::And(_) => Some(Finding::And),
            BinOp::Or(_) => Some(Finding::Or),
            _ => None,
        }
    }

    pub fn negating(operator: UnOp) -> Option<Finding> {
        match operator {
            UnOp::Not(_) => Some(Finding::Not),
            _ => None,
        }
    }

    pub fn rule(self) -> Rule {
        match self {
            Finding::If
            | Finding::IfLet
            | Finding::Match
            | Finding::While
            | Finding::For
            | Finding::Loop
            | Finding::LetElse => Rule::LinearBodies,
            Finding::Guard | Finding::ValuePattern | Finding::ArmLogic => Rule::PureRouters,
            Finding::And | Finding::Or | Finding::Not => Rule::PlainConditions,
            Finding::IgnoredElement => Rule::IndependentSelections,
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::If => formatter.write_str("if"),
            Finding::IfLet => formatter.write_str("if let that is not the whole body"),
            Finding::Match => formatter.write_str("match that is not the whole body"),
            Finding::While => formatter.write_str("while loop"),
            Finding::For => formatter.write_str("for loop"),
            Finding::Loop => formatter.write_str("loop"),
            Finding::LetElse => formatter.write_str("let-else"),
            Finding::Guard => formatter.write_str("match guard in a router"),
            Finding::ValuePattern => formatter.write_str("router arm matches a literal, range, tuple or slice"),
            Finding::ArmLogic => formatter.write_str("router arm is not a single call or value"),
            Finding::And => formatter.write_str("boolean operator &&"),
            Finding::Or => formatter.write_str("boolean operator ||"),
            Finding::Not => formatter.write_str("boolean operator !"),
            Finding::IgnoredElement => formatter.write_str("selection whose predicate does not read its element"),
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        match self {
            Finding::If => Fix::new(
                "turn the whole body into a pure router (one match on an enum, Option or Result), or give the decision a named outcome (an enum a pure router matches)",
            ),
            Finding::IfLet => Fix::new(
                "extract the if let into a pure router (a function whose whole body is that if let), or give the decision a named outcome (an enum a pure router matches)",
            ),
            Finding::Match => Fix::new(
                "extract the match into a pure router (a function whose whole body is that match), or give the decision a named outcome (an enum a pure router matches)",
            ),
            Finding::While | Finding::For => Fix::new(Self::COMBINATORS),
            Finding::Loop => {
                Fix::new("drive an iterator with combinators (successors, repeat_with, take_while, try_fold)")
            }
            Finding::LetElse => Fix::new(
                "extract a pure router (a function whose whole body is one match on the Option or Result), or give the decision a named outcome (an enum a pure router matches)",
            ),
            Finding::Guard => Fix::new("name the condition as an enum or a predicate type and match on it"),
            Finding::ValuePattern => Fix::new("match on an enum, Option or Result that names the cases"),
            Finding::ArmLogic => Fix::new("extract the arm's logic into its own linear function and call it"),
            Finding::And | Finding::Or | Finding::Not => Fix::new(Self::CONDITION),
            Finding::IgnoredElement => Fix::new(
                "a predicate that ignores its element is a branch: give the decision a named outcome, an enum a pure router matches",
            ),
        }
    }
}
