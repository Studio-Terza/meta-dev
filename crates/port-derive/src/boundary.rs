use proc_macro2::Span;
use quote::ToTokens;
use syn::{Ident, LitStr};

#[derive(Clone, Copy, Debug)]
pub enum Derivation {
    Filler,
    OpsModule,
}

#[derive(Clone, Copy, Debug)]
pub enum Naming {
    Type,
    Release,
}

impl Naming {
    fn suffix(self) -> &'static str {
        match self {
            Naming::Type => "",
            Naming::Release => "::release",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Case {
    Raise,
    Keep,
}

impl Case {
    fn after(letter: char) -> Case {
        Some(letter).filter(|next| *next == '_').map_or(Case::Keep, |_| Case::Raise)
    }

    fn applied(self, letter: char) -> Vec<char> {
        match self {
            Case::Raise => letter.to_uppercase().collect(),
            Case::Keep => vec![letter],
        }
    }
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
        let snake: String = caller
            .to_string()
            .chars()
            .enumerate()
            .flat_map(|(position, letter)| {
                let separator = (position > 0).then_some(letter).filter(|next| next.is_uppercase()).map(|_| '_');
                separator.into_iter().chain(letter.to_lowercase())
            })
            .collect();
        format!("{snake}_ops")
    }

    pub fn camel(method: &Ident) -> Ident {
        let camel: String = method
            .to_string()
            .chars()
            .scan(Case::Raise, |case, letter| {
                let current = *case;
                *case = Case::after(letter);
                Some((current, letter))
            })
            .filter(|(_, letter)| *letter != '_')
            .flat_map(|(case, letter)| case.applied(letter))
            .collect();
        Ident::new(&camel, method.span())
    }

    pub fn method_path(caller: &impl ToTokens, method: &Ident) -> LitStr {
        LitStr::new(&format!("::{}::{method}", caller.to_token_stream()), method.span())
    }

    pub fn type_path(ident: &Ident, naming: Naming) -> LitStr {
        LitStr::new(&format!("::{ident}{}", naming.suffix()), Span::call_site())
    }
}
