use syn::{Block, Expr, ExprIf, ExprMatch, Macro, Stmt};

use lint_layout::{Name, Whole, Word, words};

use super::super::trace::{Evaluation, Trace};
use crate::boundary::Truth;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Flow {
    Continues,
    Ends,
}

pub(crate) struct Reach<'trace, 'body, 'file> {
    trace: &'trace Trace<'body, 'file>,
}

impl<'trace, 'body, 'file> Reach<'trace, 'body, 'file> {
    const STOPS: [Word<'static>; 4] = words!["panic", "unreachable", "todo", "unimplemented"];

    pub(crate) fn after(trace: &'trace Trace<'body, 'file>, statement: &Stmt) -> Flow {
        Reach { trace }.statement(statement)
    }

    pub(crate) fn statement(&self, statement: &Stmt) -> Flow {
        match statement {
            Stmt::Expr(expression, _) => self.expression(expression),
            Stmt::Macro(invocation) => Self::stopping(&invocation.mac),
            Stmt::Local(_) | Stmt::Item(_) => Flow::Continues,
        }
    }

    pub(crate) fn expression(&self, expression: &Expr) -> Flow {
        match expression {
            Expr::Return(_) | Expr::Break(_) | Expr::Continue(_) => Flow::Ends,
            Expr::Macro(invocation) => Self::stopping(&invocation.mac),
            Expr::Paren(inner) => self.expression(&inner.expr),
            Expr::Block(inner) => self.block(&inner.block),
            Expr::Unsafe(inner) => self.block(&inner.block),
            Expr::If(branch) => self.branches(branch),
            Expr::Match(matched) => self.arms(matched),
            _ => Flow::Continues,
        }
    }

    pub(crate) fn block(&self, block: &Block) -> Flow {
        block.stmts.iter().map(|statement| self.statement(statement)).max().unwrap_or(Flow::Continues)
    }

    pub(crate) fn arms(&self, matched: &ExprMatch) -> Flow {
        matched.arms.iter().map(|arm| self.expression(&arm.body)).min().unwrap_or(Flow::Continues)
    }

    pub(crate) fn branches(&self, branch: &ExprIf) -> Flow {
        let then = self.block(&branch.then_branch);
        let other = branch.else_branch.as_ref().map_or(Flow::Continues, |(_, other)| self.expression(other));
        Self::chosen(Evaluation::condition_of(self.trace, &branch.cond), then, other)
    }

    fn chosen(truth: Option<Truth>, then: Flow, other: Flow) -> Flow {
        match truth {
            Some(Truth::Holds) => then,
            Some(Truth::Fails) => other,
            None => then.min(other),
        }
    }

    fn stopping(invocation: &Macro) -> Flow {
        invocation
            .path
            .segments
            .last()
            .and_then(|last| Name::from(&last.ident).matches::<Whole>(&Self::STOPS))
            .map_or(Flow::Continues, |_| Flow::Ends)
    }
}
