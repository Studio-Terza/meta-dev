use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, Type};

pub(crate) struct Parameter {
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
    pub(crate) fn named(name: Ident, tree: &Type) -> Parameter {
        Parameter { name, tree: tree.clone() }
    }

    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    pub(crate) fn tree(&self) -> &Type {
        &self.tree
    }
}
