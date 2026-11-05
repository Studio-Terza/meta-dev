use crate::named::boundary::Accessor;
use syn::parse::{Parse, ParseStream};
use syn::{Error, Ident, Lit, Token};

pub struct Value {
    accessor: Ident,
    lit: Lit,
}

impl Value {
    const UNDECLARED: &'static str = "no #[accessor(name: type)] on the enum declares this accessor";

    pub fn declared_in(&self, accessors: &[Accessor]) -> syn::Result<()> {
        accessors
            .iter()
            .find(|accessor| accessor.name() == &self.accessor)
            .ok_or_else(|| Error::new_spanned(&self.accessor, Self::UNDECLARED))
            .and_then(|accessor| accessor.checked(&self.lit))
            .map(|_| ())
    }

    pub fn accessor(&self) -> &Ident {
        &self.accessor
    }

    pub fn lit(&self) -> &Lit {
        &self.lit
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Value> {
        let accessor = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let lit = input.parse::<Lit>()?;
        Ok(Value { accessor, lit })
    }
}
