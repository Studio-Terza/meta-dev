use proc_macro2::TokenStream;
use quote::quote;

use super::boundary::Naming;
use super::named::Named;

pub struct OpDerive;

impl OpDerive {
    pub fn expanded(tokens: TokenStream) -> TokenStream {
        Named::parse(tokens).map_or_else(
            |error| error.to_compile_error(),
            |named| {
                let name = named.op_name(Naming::Type);
                named.implementing(&quote! { ::port_core::Op }, &quote! { const NAME: ::port_core::OpName = #name; })
            },
        )
    }
}
