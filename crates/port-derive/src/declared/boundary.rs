use grammar::ident::Words;
use quote::{ToTokens, format_ident};
use syn::{Ident, LitStr};

#[derive(Clone, Copy, Debug)]
pub enum Derivation {
    Filler,
    OpsModule,
}

pub struct Spelling;

impl Spelling {
    pub fn derived(caller: &Ident, derivation: Derivation) -> Ident {
        match derivation {
            Derivation::Filler => Ident::new(&format!("{caller}Filler"), caller.span()),
            Derivation::OpsModule => Ident::new(&Self::ops_module_text(caller), caller.span()),
        }
    }

    fn ops_module_text(caller: &Ident) -> String {
        format!("{}_ops", Words::lowered_humps(&caller.to_string()).join("_"))
    }

    pub fn camel(method: &Ident) -> Ident {
        Ident::new(&Words::camel(&method.to_string()), method.span())
    }

    pub fn input(position: usize) -> Ident {
        format_ident!("input{position}")
    }

    pub fn method_path(caller: &impl ToTokens, method: &Ident) -> LitStr {
        LitStr::new(&format!("::{}::{method}", caller.to_token_stream()), method.span())
    }
}
