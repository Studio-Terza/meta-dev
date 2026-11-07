use syn::{Expr, ExprStruct, ImplItemFn, Stmt};

use lint_core::Syntax;
use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MethodShape {
    Constructor,
    Accessor,
    Logic,
}

impl MethodShape {
    const SELF: &str = "Self";

    pub fn classify(function: &ImplItemFn, self_name: &Name) -> Self {
        Some(function.block.stmts.as_slice())
            .filter(|statements| statements.len() == 1)
            .and_then(<[Stmt]>::first)
            .and_then(Self::tail)
            .map_or(Self::Logic, |expr| Self::of_expression(expr, self_name))
    }

    fn tail(statement: &Stmt) -> Option<&Expr> {
        match statement {
            Stmt::Expr(expr, None) => Some(expr),
            _ => None,
        }
    }

    fn of_expression(expr: &Expr, self_name: &Name) -> Self {
        match expr {
            Expr::Field(field) => Syntax::expr_path(&field.base)
                .filter(|base| base.path.is_ident("self"))
                .map_or(Self::Logic, |_| Self::Accessor),
            Expr::Struct(literal) => Self::of_literal(literal, self_name),
            _ => Self::Logic,
        }
    }

    fn of_literal(literal: &ExprStruct, self_name: &Name) -> Self {
        let last = literal.path.segments.last();
        last.filter(|segment| segment.ident == Self::SELF)
            .or_else(|| last.filter(|segment| *self_name == Name::from(&segment.ident)))
            .map_or(Self::Logic, |_| Self::Constructor)
    }
}
