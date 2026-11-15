use syntax::parse::{Parse, ParseStream};
use syntax::{Ident, Token};

use lint_core::TypeName;

pub struct Transition {
    target: TypeName,
}

impl Transition {
    pub fn target(&self) -> &TypeName {
        &self.target
    }
}

impl Parse for Transition {
    fn parse(input: ParseStream) -> syntax::Result<Transition> {
        input.parse::<Ident>()?;
        input.parse::<Token![->]>()?;
        Ok(Transition { target: TypeName::named(&input.parse()?) })
    }
}
