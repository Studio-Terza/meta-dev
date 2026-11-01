use syn::{Expr, ExprStruct, ImplItemFn, Stmt};

use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MethodShape {
    Constructor,
    Accessor,
    Logic,
}

impl MethodShape {
    const SELF: &str = "Self";

    pub fn classify(function: &ImplItemFn, self_name: &Name) -> MethodShape {
        match function.block.stmts.as_slice() {
            [Stmt::Expr(expr, None)] => Self::of_expression(expr, self_name),
            _ => MethodShape::Logic,
        }
    }

    fn of_expression(expr: &Expr, self_name: &Name) -> MethodShape {
        match expr {
            Expr::Field(field) if matches!(&*field.base, Expr::Path(base) if base.path.is_ident("self")) => {
                MethodShape::Accessor
            }
            Expr::Struct(literal) => Self::of_literal(literal, self_name),
            _ => MethodShape::Logic,
        }
    }

    fn of_literal(literal: &ExprStruct, self_name: &Name) -> MethodShape {
        match literal.path.segments.last() {
            Some(segment) if segment.ident == Self::SELF || *self_name == segment.ident => MethodShape::Constructor,
            _ => MethodShape::Logic,
        }
    }
}
