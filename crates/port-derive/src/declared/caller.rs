use proc_macro2::TokenStream;
use quote::quote;

use super::method::Method;
use super::{Declared, Rendered};

pub struct Caller;

impl Rendered for Caller {
    fn rendered(declared: &Declared) -> TokenStream {
        let attributes = declared.attributes();
        let visibility = declared.visibility();
        let name = declared.caller_name();
        let items = declared.as_ref().iter().map(Self::method_item);
        quote! {
            #(#attributes)*
            #visibility trait #name: ::port_core::Metered {
                #(#items)*
            }
        }
    }
}

impl Caller {
    fn method_item(method: &Method) -> TokenStream {
        let attributes = method.attributes();
        let signature = method.signature();
        method
            .provided()
            .map_or_else(|| quote! { #(#attributes)* #signature; }, |body| quote! { #(#attributes)* #signature #body })
    }
}
