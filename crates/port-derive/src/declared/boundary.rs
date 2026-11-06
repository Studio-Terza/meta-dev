use grammar::ident::Words;
use proc_macro2::Span;
use quote::{ToTokens, format_ident};
use syn::{Ident, LitStr};

#[derive(Clone, Copy, Debug)]
pub enum Derivation {
    Filler,
    OpsModule,
}

#[derive(Clone, Copy, Debug)]
pub enum Constant {
    Name,
    Release,
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

    pub fn constant(constant: Constant) -> Ident {
        match constant {
            Constant::Name => Ident::new("NAME", Span::call_site()),
            Constant::Release => Ident::new("RELEASE", Span::call_site()),
        }
    }

    pub fn type_path(ident: &Ident, constant: Constant) -> LitStr {
        match constant {
            Constant::Name => LitStr::new(&format!("::{ident}"), ident.span()),
            Constant::Release => LitStr::new(&format!("::{ident}::release"), ident.span()),
        }
    }
}
