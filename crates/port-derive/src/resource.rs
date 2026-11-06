use proc_macro2::TokenStream;
use quote::quote;

use super::boundary::Naming;
use super::named::Named;

pub struct ResourceDerive;

impl ResourceDerive {
    pub fn expanded(tokens: TokenStream) -> TokenStream {
        Named::parse(tokens).map_or_else(
            |error| error.to_compile_error(),
            |named| {
                let name = named.op_name(Naming::Type);
                let release = named.op_name(Naming::Release);
                named.implementing(
                    &quote! { ::port_core::Resource },
                    &quote! {
                        const NAME: ::port_core::OpName = #name;
                        const RELEASE: ::port_core::OpName = #release;
                    },
                )
            },
        )
    }
}
