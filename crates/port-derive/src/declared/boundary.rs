use proc_macro2::{Ident, Span};
use quote::{ToTokens, format_ident};
use syn::LitStr;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Derivation {
    Filler,
    OpsModule,
}

#[derive(strum::VariantArray, Clone, Copy, Debug)]
pub(crate) enum Constant {
    Name,
    Release,
}

pub(crate) struct Spelling;

impl Spelling {
    pub(crate) fn derived(caller: &Ident, derivation: Derivation) -> Ident {
        match derivation {
            Derivation::Filler => Ident::new(&format!("{caller}Filler"), caller.span()),
            Derivation::OpsModule => Ident::new(&Self::ops_module_text(caller), caller.span()),
        }
    }

    fn ops_module_text(caller: &Ident) -> String {
        let text = caller.to_string();
        let gaps =
            std::iter::once(None).chain(text.chars().skip(1).map(|letter| letter.is_ascii_uppercase().then_some('_')));
        let snake: String =
            text.chars().zip(gaps).flat_map(|(letter, gap)| gap.into_iter().chain(letter.to_lowercase())).collect();
        format!("{snake}_ops")
    }

    pub(crate) fn camel(method: &Ident) -> Ident {
        let text = method.to_string();
        let raised = std::iter::once('_').chain(text.chars()).map(|before| before == '_');
        let camel: String = text
            .chars()
            .zip(raised)
            .filter(|(letter, _)| *letter != '_')
            .flat_map(|(letter, upper)| if upper { letter.to_uppercase().collect::<Vec<char>>() } else { vec![letter] })
            .collect();
        Ident::new(&camel, method.span())
    }

    pub(crate) fn input(position: usize) -> Ident {
        format_ident!("input{position}")
    }

    pub(crate) fn method_path(caller: &impl ToTokens, method: &Ident) -> LitStr {
        LitStr::new(&format!("::{}::{method}", caller.to_token_stream()), method.span())
    }

    pub(crate) fn constant(constant: Constant) -> Ident {
        match constant {
            Constant::Name => Ident::new("NAME", Span::call_site()),
            Constant::Release => Ident::new("RELEASE", Span::call_site()),
        }
    }

    pub(crate) fn type_path(ident: &Ident, constant: Constant) -> LitStr {
        match constant {
            Constant::Name => LitStr::new(&format!("::{ident}"), ident.span()),
            Constant::Release => LitStr::new(&format!("::{ident}::release"), ident.span()),
        }
    }
}
