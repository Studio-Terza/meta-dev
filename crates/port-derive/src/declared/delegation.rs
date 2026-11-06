use proc_macro2::TokenStream;
use quote::quote;

use super::method::{Method, Parameter};
use super::{Declared, Rendered};

pub struct Delegation;

impl Rendered for Delegation {
    fn rendered(declared: &Declared) -> TokenStream {
        let name = declared.caller_name();
        let filler = declared.filler_name();
        let items = declared.methods().iter().filter(|method| method.provided().is_none()).map(Self::delegated_item);
        quote! {
            impl<'run, F: #filler> #name for ::port_core::Port<'run, F> {
                #(#items)*
            }
        }
    }
}

impl Delegation {
    fn delegated_item(method: &Method) -> TokenStream {
        let signature = method.signature();
        let counting = method.counting();
        let method_name = &signature.ident;
        let arguments = method.parameters().iter().map(Parameter::name);
        quote! {
            #signature {
                self.#counting(|filler: &F, pass| filler.#method_name(pass #(, #arguments)*))
            }
        }
    }
}
