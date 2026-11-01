use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token};

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
    fn parse(input: ParseStream) -> syn::Result<Transition> {
        input.parse::<Ident>()?;
        input.parse::<Token![->]>()?;
        Ok(Transition { target: TypeName::named(&input.parse()?) })
    }
}
