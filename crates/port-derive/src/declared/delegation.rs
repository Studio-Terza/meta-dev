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
        let parameters = method.parameters();
        let arguments = parameters.iter().map(Parameter::name);
        let output = method.output();
        quote! {
            fn #name(&self #(, #parameters)*) #output {
                self.counted_call(|filler: &F, pass| filler.#name(pass #(, #arguments)*))
            }
        }
    }
}
