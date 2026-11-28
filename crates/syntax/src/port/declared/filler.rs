use proc_macro2::TokenStream;
use quote::quote;

use super::definition::Declared;
use super::method::Method;
use super::names::OpsName;

pub struct Filler;

impl Filler {
    pub fn rendered(declared: &Declared) -> TokenStream {
        let visibility = declared.visibility();
        let filler = declared.filler_name();
        let ops = declared.ops_name();
        let items = declared.as_ref().iter().map(|method| Self::required_item(method, ops));
        quote! {
            #visibility trait #filler {
                #(#items)*
            }
        }
    }

    fn required_item(method: &Method, ops: &OpsName) -> TokenStream {
        let name = method.name();
        let generics = method.generics();
        let bounds = &generics.where_clause;
        let pass = method.pass(ops);
        let parameters = method.parameters();
        let output = method.returned().filler_output(method.output());
        quote! { fn #name #generics (&self, pass: #pass #(, #parameters)*) #output #bounds; }
    }
}
