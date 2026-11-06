use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Type};

pub struct Parameter {
    name: Ident,
    tree: Type,
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let tree = &self.tree;
        tokens.extend(quote! { #name: #tree });
    }
}

impl Parameter {
    pub fn named(name: Ident, tree: &Type) -> Parameter {
        Parameter { name, tree: tree.clone() }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
