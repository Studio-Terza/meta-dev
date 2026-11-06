use logos::Logos;
use quote::{ToTokens, format_ident};
use syn::{Ident, LitStr};

#[derive(Clone, Copy, Debug)]
pub enum Derivation {
    Filler,
    OpsModule,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Hump {
    #[regex(r"[A-Z][^A-Z]*")]
    Capitalized,
    #[regex(r"[^A-Z]+")]
    Leading,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Snake {
    #[regex(r"[^_]+")]
    Word,
    #[token("_")]
    Underscore,
}

impl Snake {
    fn raised(piece: Result<Snake, ()>, written: &str) -> Vec<char> {
        match piece {
            Ok(Snake::Word) => Snake::capitalized(written),
            Ok(Snake::Underscore) | Err(()) => Vec::new(),
        }
    }

    fn capitalized(word: &str) -> Vec<char> {
        word.chars().take(1).flat_map(char::to_uppercase).chain(word.chars().skip(1)).collect()
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
        let text = caller.to_string();
        let words: Vec<String> = Hump::lexer(&text)
            .spanned()
            .filter_map(|(_, range)| text.get(range))
            .map(|word| word.chars().flat_map(char::to_lowercase).collect())
            .collect();
        format!("{}_ops", words.join("_"))
    }

    pub fn camel(method: &Ident) -> Ident {
        let text = method.to_string();
        let camel: String = Snake::lexer(&text)
            .spanned()
            .flat_map(|(piece, range)| Snake::raised(piece, text.get(range).unwrap_or_default()))
            .collect();
        Ident::new(&camel, method.span())
    }

    pub fn input(position: usize) -> Ident {
        format_ident!("input{position}")
    }

    pub fn method_path(caller: &impl ToTokens, method: &Ident) -> LitStr {
        LitStr::new(&format!("::{}::{method}", caller.to_token_stream()), method.span())
    }
}
