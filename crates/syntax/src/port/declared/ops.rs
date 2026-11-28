use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::boundary::Spelling;
use super::definition::Declared;
use super::method::Method;
use super::names::CallerName;

pub struct Ops;

impl Ops {
    pub fn rendered(declared: &Declared) -> TokenStream {
        let visibility = declared.visibility();
        let module = declared.ops_name();
        let caller = declared.caller_name();
        let declarations = declared
            .as_ref()
            .iter()
            .filter_map(|method| method.op_struct().map(|op| Self::op_declaration(&op, caller, method)));
        quote! {
            #visibility mod #module {
                #(#declarations)*
            }
        }
    }

    fn op_declaration(op: &Ident, caller: &CallerName, method: &Method) -> TokenStream {
        let path = Spelling::method_path(caller, method.name());
        let name = quote!(::port_core::OpName);
        quote! {
            #[derive(Debug)]
            pub struct #op;

            impl ::port_core::Op for #op {
                const NAME: #name = #name::new(concat!(module_path!(), #path));
            }
        }
    }
}
