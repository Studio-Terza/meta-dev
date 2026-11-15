use syntax::Literal;

use super::Spelling;

impl From<&Literal> for Spelling {
    fn from(literal: &Literal) -> Spelling {
        Spelling::new(literal.to_string())
    }
}
