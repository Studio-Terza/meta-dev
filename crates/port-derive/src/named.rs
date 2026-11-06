use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident};

use super::declared::{Constant, Spelling};

#[derive(Clone, Copy, Debug)]
pub enum Derived {
    Op,
    Resource,
}

pub struct Named {
    ident: Ident,
    generics: Generics,
}

impl Derived {
    const OP: [Constant; 1] = [Constant::Name];
    const RESOURCE: [Constant; 2] = [Constant::Name, Constant::Release];

    fn implemented(self) -> TokenStream {
        match self {
            Derived::Op => quote!(::port_core::Op),
            Derived::Resource => quote!(::port_core::Resource),
        }
    }

    fn constants(self) -> &'static [Constant] {
        match self {
            Derived::Op => &Self::OP,
            Derived::Resource => &Self::RESOURCE,
        }
    }
}

impl Named {
    pub fn expanded(tokens: TokenStream, derived: Derived) -> TokenStream {
        syn::parse2::<DeriveInput>(tokens)
            .map(|DeriveInput { ident, generics, .. }| Named { ident, generics })
            .map_or_else(|error| error.to_compile_error(), |named| named.implementing(derived))
    }

    fn implementing(&self, derived: Derived) -> TokenStream {
        let name = &self.ident;
        let (parameters, arguments, bounds) = self.generics.split_for_impl();
        let implemented = derived.implemented();
        let constants = derived.constants().iter().map(|constant| self.constant(*constant));
        quote! {
            impl #parameters #implemented for #name #arguments #bounds {
                #(#constants)*
            }
        }
    }

    fn constant(&self, constant: Constant) -> TokenStream {
        let declared = Spelling::constant(constant);
        let path = Spelling::type_path(&self.ident, constant);
        quote! { const #declared: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), #path)); }
    }
}
