use syn::{Block, Expr, ExprField, ExprPath, Stmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    FieldProjection,
    Other,
}

impl Body {
    const RECEIVER: &'static str = "self";

    pub fn classify(block: &Block) -> Self {
        block
            .stmts
            .first()
            .filter(|_| block.stmts.len() == 1)
            .and_then(Self::field_of)
            .and_then(|field| Self::path_of(&field.base))
            .filter(|base| base.path.is_ident(Self::RECEIVER))
            .map_or(Self::Other, |_| Self::FieldProjection)
    }

    fn field_of(statement: &Stmt) -> Option<&ExprField> {
        match statement {
            Stmt::Expr(Expr::Field(field), None) => Some(field),
            _ => None,
        }
    }

    fn path_of(base: &Expr) -> Option<&ExprPath> {
        match base {
            Expr::Path(path) => Some(path),
            _ => None,
        }
    }
}
