use syn::{Block, Expr, ExprIf, ExprMatch, Macro, Stmt};

use lint_layout::{Name, Whole, Word, words};

use super::super::trace::{Evaluation, Lowering, Trace};
use crate::boundary::{Spelling, Truth};
use crate::claim::{Callee, Premise, Term};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Flow {
    Falls,
    Skips,
    Leaves,
}

impl Flow {
    fn stops(self) -> Option<Flow> {
        Some(self).filter(|flow| *flow != Flow::Falls)
    }

    pub(crate) fn only(self, expected: Flow) -> Option<Flow> {
        Some(self).filter(|flow| *flow == expected)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Alternatives {
    then: Flow,
    other: Flow,
}

pub(crate) struct Reach<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
}

impl<'trace, 'body, 'file> Reach<'trace, 'body, 'file> {
    const STOPS: [Word<'static>; 4] = words!["panic", "unreachable", "todo", "unimplemented"];

    pub(crate) fn after(trace: &'trace Trace<'body, 'file>, statement: &Stmt) -> Flow {
        Reach { trace }.statement(statement)
    }

    pub(crate) fn within(trace: &'trace Trace<'body, 'file>, block: &Block) -> Flow {
        Reach { trace }.block(block)
    }

    pub(crate) fn remaining(trace: &'trace Trace<'body, 'file>, statement: &Stmt) -> Vec<Premise> {
        Reach { trace }.conditions(statement)
    }

    pub(crate) fn statement(&self, statement: &Stmt) -> Flow {
        match statement {
            Stmt::Expr(expression, _) => self.expression(expression),
            Stmt::Macro(invocation) => Self::stopping(&invocation.mac),
            Stmt::Local(_) | Stmt::Item(_) => Flow::Falls,
        }
    }

    pub(crate) fn expression(&self, expression: &Expr) -> Flow {
        match expression {
            Expr::Return(_) | Expr::Break(_) => Flow::Leaves,
            Expr::Continue(_) => Flow::Skips,
            Expr::Macro(invocation) => Self::stopping(&invocation.mac),
            Expr::Paren(inner) => self.expression(&inner.expr),
            Expr::Block(inner) => self.block(&inner.block),
            Expr::Unsafe(inner) => self.block(&inner.block),
            Expr::If(branch) => self.branches(branch),
            Expr::Match(matched) => self.arms(matched),
            _ => Flow::Falls,
        }
    }

    pub(crate) fn block(&self, block: &Block) -> Flow {
        block.stmts.iter().map(|statement| self.statement(statement)).find_map(Flow::stops).unwrap_or(Flow::Falls)
    }

    pub(crate) fn arms(&self, matched: &ExprMatch) -> Flow {
        matched.arms.iter().map(|arm| self.expression(&arm.body)).min().unwrap_or(Flow::Falls)
    }

    pub(crate) fn branches(&self, branch: &ExprIf) -> Flow {
        let alternatives = self.alternatives(branch);
        Self::chosen(Evaluation::condition_of(self.trace, &branch.cond), alternatives)
    }

    pub(crate) fn alternatives(&self, branch: &ExprIf) -> Alternatives {
        Alternatives {
            then: self.block(&branch.then_branch),
            other: branch.else_branch.as_ref().map_or(Flow::Falls, |(_, other)| self.expression(other)),
        }
    }

    pub(crate) fn conditions(&self, statement: &Stmt) -> Vec<Premise> {
        match statement {
            Stmt::Expr(Expr::If(branch), _) => self.surviving(branch),
            Stmt::Expr(Expr::Match(matched), _) => self.unmatched(matched),
            Stmt::Expr(..) | Stmt::Macro(_) | Stmt::Local(_) | Stmt::Item(_) => Vec::new(),
        }
    }

    pub(crate) fn surviving(&self, branch: &ExprIf) -> Vec<Premise> {
        let alternatives = self.alternatives(branch);
        let held = alternatives
            .then
            .stops()
            .and(alternatives.other.only(Flow::Falls))
            .map(|_| Truth::Fails)
            .or_else(|| alternatives.other.stops().and(alternatives.then.only(Flow::Falls)).map(|_| Truth::Holds));
        held.filter(|_| Evaluation::condition_of(self.trace, &branch.cond).is_none())
            .map(|holds| Premise::new(Lowering::term_of(self.trace, &branch.cond), holds).conjuncts())
            .unwrap_or_default()
    }

    pub(crate) fn unmatched(&self, matched: &ExprMatch) -> Vec<Premise> {
        let scrutinee = Lowering::term_of(self.trace, &matched.expr);
        matched
            .arms
            .iter()
            .filter(|arm| self.expression(&arm.body).stops().is_some())
            .map(|arm| {
                Premise::new(
                    Callee::Function(Name::from("matches")).folded(vec![
                        scrutinee.clone(),
                        Term::Opaque(Spelling::written(&arm.pat)),
                        Term::Opaque(Spelling::written(&arm.guard.as_ref().map(|(_, guard)| guard))),
                    ]),
                    Truth::Fails,
                )
            })
            .collect()
    }

    fn chosen(truth: Option<Truth>, alternatives: Alternatives) -> Flow {
        match truth {
            Some(Truth::Holds) => alternatives.then,
            Some(Truth::Fails) => alternatives.other,
            None => alternatives.then.min(alternatives.other),
        }
    }

    fn stopping(invocation: &Macro) -> Flow {
        invocation
            .path
            .segments
            .last()
            .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Self::STOPS))
            .map_or(Flow::Falls, |_| Flow::Leaves)
    }
}
