use syn::{Block, Expr, Stmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    FieldProjection,
    Other,
}

impl Body {
    const RECEIVER: &'static str = "self";

    pub fn classify(block: &Block) -> Body {
        match block.stmts.as_slice() {
            [Stmt::Expr(Expr::Field(field), None)] if matches!(&*field.base, Expr::Path(base) if base.path.is_ident(Self::RECEIVER)) => {
                Body::FieldProjection
            }
            _ => Body::Other,
        }
    }
}
