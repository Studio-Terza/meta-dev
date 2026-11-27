use proc_macro2::TokenStream;
use quote::quote;

use super::method::{Method, Parameter};
use super::{Declared, Rendered};

pub struct Delegation;

impl Rendered for Delegation {
    fn rendered(declared: &Declared) -> TokenStream {
        let name = declared.caller_name();
        let filler = declared.filler_name();
        let items = declared.as_ref().iter().map(Self::delegated_item);
        quote! {
            impl<'run, F: #filler> #name for ::port_core::Port<'run, F> {
                #(#items)*
            }
        }
    }
}

impl Delegation {
    fn delegated_item(method: &Method) -> TokenStream {
        let name = method.name();
        let generics = method.generics();
        let bounds = &generics.where_clause;
        let parameters = method.parameters();
        let arguments = parameters.iter().map(Parameter::name);
        let output = method.output();
        let counting = method.returned().counting();
        quote! {
            fn #name #generics (&self #(, #parameters)*) #output #bounds {
                self.#counting(|filler: &F, pass| filler.#name(pass #(, #arguments)*))
            }
        }
    }
}
