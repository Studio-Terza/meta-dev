use strum::{Display, VariantArray};
use syntax::{BinOp, Expr, UnOp};

use lint_core::Fix;
use lint_sweep::Member;

#[derive(VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    LinearBodies,
    PureRouters,
    PlainConditions,
}

impl Rule {
    const ROUTERS: Member = Member::new(1);
    const CONDITIONS: Member = Member::new(2);

    pub const fn member(self) -> Member {
        match self {
            Rule::LinearBodies => Member::FIRST,
            Rule::PureRouters => Self::ROUTERS,
            Rule::PlainConditions => Self::CONDITIONS,
        }
    }
}

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finding {
    #[strum(serialize = "if")]
    If,
    #[strum(serialize = "if let that is not the whole body")]
    IfLet,
    #[strum(serialize = "match that is not the whole body")]
    Match,
    #[strum(serialize = "while loop")]
    While,
    #[strum(serialize = "for loop")]
    For,
    #[strum(serialize = "loop")]
    Loop,
    #[strum(serialize = "let-else")]
    LetElse,
    #[strum(serialize = "match guard in a router")]
    Guard,
    #[strum(serialize = "router arm matches a literal, range, tuple or slice")]
    ValuePattern,
    #[strum(serialize = "router arm is not a single call or value")]
    ArmLogic,
    #[strum(serialize = "boolean operator &&")]
    And,
    #[strum(serialize = "boolean operator ||")]
    Or,
    #[strum(serialize = "boolean operator !")]
    Not,
}

impl Finding {
    const COMBINATORS: &'static str = "drive an iterator with combinators (map, filter, fold, try_fold, take_while)";
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
        }
    }
}

impl lint_core::Finding for Finding {
    fn fix(&self) -> Fix {
        match self {
            Finding::If => Fix::new(
                "use combinators on Option, Result and iterators (then_some, filter, map_or), or make the whole body one match on an enum, Option or Result",
            ),
            Finding::IfLet => Fix::new(
                "use combinators on Option and Result (map, and_then, map_or), or extract it into its own function whose whole body is the if let",
            ),
            Finding::Match => Fix::new(
                "extract it into its own function whose whole body is the match, or use combinators on Option, Result and iterators",
            ),
            Finding::While | Finding::For => Fix::new(Self::COMBINATORS),
            Finding::Loop => {
                Fix::new("drive an iterator with combinators (successors, repeat_with, take_while, try_fold)")
            }
            Finding::LetElse => Fix::new("use combinators on Option and Result, or ? after ok_or"),
            Finding::Guard => Fix::new("name the condition as an enum or a predicate type and match on it"),
            Finding::ValuePattern => Fix::new("match on an enum, Option or Result that names the cases"),
            Finding::ArmLogic => Fix::new("extract the arm's logic into its own linear function and call it"),
            Finding::And | Finding::Or | Finding::Not => Fix::new(Self::CONDITION),
        }
    }
}
