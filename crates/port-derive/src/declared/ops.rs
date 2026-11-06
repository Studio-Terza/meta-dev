use proc_macro2::TokenStream;
use quote::quote;

use super::boundary::Spelling;
use super::method::Method;
use super::names::CallerName;
use super::{Declared, Rendered};

pub struct Ops;

impl Rendered for Ops {
    fn rendered(declared: &Declared) -> TokenStream {
        let visibility = declared.visibility();
        let module = declared.ops_name();
        let caller = declared.caller_name();
        let declarations = declared.as_ref().iter().map(|method| Self::op_declaration(caller, method));
        quote! {
            #visibility mod #module {
                #(#declarations)*
            }
        }
    }
}

impl Ops {
    fn op_declaration(caller: &CallerName, method: &Method) -> TokenStream {
        let op = Spelling::camel(method.name());
        let path = Spelling::method_path(caller, method.name());
        quote! {
            #[derive(Debug)]
            pub struct #op;

            impl ::port_core::Op for #op {
                const NAME: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), #path));
            }
        }
    }
}
