use proc_macro2::TokenStream;
use quote::quote;

use super::method::Method;
use super::names::OpsName;
use super::{Declared, Rendered};

pub struct Filler;

impl Rendered for Filler {
    fn rendered(declared: &Declared) -> TokenStream {
        let visibility = declared.visibility();
        let filler = declared.filler_name();
        let ops = declared.ops_name();
        let items = declared
            .as_ref()
            .iter()
            .filter(|method| method.provided().is_none())
            .map(|method| Self::required_item(method, ops));
        quote! {
            #visibility trait #filler {
                #(#items)*
            }
        }
    }
}

impl Filler {
    fn required_item(method: &Method, ops: &OpsName) -> TokenStream {
        let signature = method.signature();
        let name = &signature.ident;
        let generics = &signature.generics;
        let where_clause = &signature.generics.where_clause;
        let pass = method.pass(ops);
        let parameters = method.parameters();
        let output = &signature.output;
        quote! { fn #name #generics (&self, pass: #pass #(, #parameters)*) #output #where_clause; }
    }
}
