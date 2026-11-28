use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Generics, Ident};

use super::declared::{Constant, Spelling};
use crate::{Expansion, Input};

#[derive(Clone, Copy, Debug)]
enum Derived {
    Op,
    Resource,
}

#[derive(Debug)]
pub struct Op;

#[derive(Debug)]
pub struct Resource;

#[derive(Debug)]
struct Named {
    ident: Ident,
    generics: Generics,
}

impl Derived {
    const OP: [Constant; 1] = [Constant::Name];

    fn implemented(self) -> TokenStream {
        match self {
            Derived::Op => quote!(::port_core::Op),
            Derived::Resource => quote!(::port_core::Resource),
        }
    }

    fn constants(self) -> &'static [Constant] {
        match self {
            Derived::Op => &Self::OP,
            Derived::Resource => <Constant as strum::VariantArray>::VARIANTS,
        }
    }
}

impl<Kind> From<Input<Kind>> for Named {
    fn from(input: Input<Kind>) -> Named {
        let DeriveInput { ident, generics, .. } = input.into_inner();
        Named { ident, generics }
    }
}

impl Expansion for Input<Op> {
    fn expansion(self) -> syn::Result<TokenStream> {
        Ok(Named::from(self).implementing(Derived::Op))
    }
}

impl Expansion for Input<Resource> {
    fn expansion(self) -> syn::Result<TokenStream> {
        Ok(Named::from(self).implementing(Derived::Resource))
    }
}

impl Named {
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
