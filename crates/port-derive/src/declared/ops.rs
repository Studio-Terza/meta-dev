use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use super::method::Method;
use super::names::CallerName;
use super::{Declared, Rendered};
use crate::boundary::Spelling;

pub struct Ops;

impl Rendered for Ops {
    fn rendered(declared: &Declared) -> TokenStream {
        let visibility = declared.visibility();
        let module = declared.ops_name();
        let caller = declared.caller_name();
        let declarations = declared
            .methods()
            .iter()
            .filter(|method| method.provided().is_none())
            .filter_map(|method| method.op_struct().map(|op| Self::op_declaration(&op, caller, method)));
        quote! {
            #visibility mod #module {
                #(#declarations)*
            }
        }
    }
}

impl Ops {
    fn op_declaration(op: &Ident, caller: &CallerName, method: &Method) -> TokenStream {
        let path = Spelling::method_path(caller, &method.signature().ident);
        quote! {
            #[derive(Debug)]
            pub struct #op;

            impl ::port_core::Op for #op {
                const NAME: ::port_core::OpName = ::port_core::OpName::new(concat!(module_path!(), #path));
            }
        }
    }
}
