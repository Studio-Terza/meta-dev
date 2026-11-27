use proc_macro2::{TokenStream, TokenTree};
use syn::{Macro, TypeTraitObject};

use crate::finding::{Found, Rule};
use crate::source::Source;
use crate::style::{Reads, Style};
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Debug, Default)]
pub(crate) struct NoDyn {
    found: Vec<Found>,
}

impl NoDyn {
    const KEYWORD: &'static str = "dyn";

    fn written(tokens: TokenStream) -> Vec<Found> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Group(group) => NoDyn::written(group.stream()),
                TokenTree::Ident(ident) if ident == NoDyn::KEYWORD => vec![NoDyn::flagged(ident.span())],
                TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
            })
            .collect()
    }

    fn flagged(span: proc_macro2::Span) -> Found {
        Found::spanned(
            span,
            "dyn trait object",
            "take a generic, return impl Trait, or list the known types in an enum",
        )
    }
}

impl Hooks for NoDyn {
    fn type_trait_object(&mut self, object: &TypeTraitObject) -> Descent {
        self.found.extend(object.dyn_token.map(|keyword| NoDyn::flagged(keyword.span)));
        Descent::Into
    }

    fn macro_called(&mut self, invocation: &Macro) -> Expanded {
        self.found.extend(NoDyn::written(invocation.tokens.clone()));
        Expanded::Nothing
    }
}

impl<'file> Style<'file> for NoDyn {
    const RULE: Rule = Rule::NoDyn;
    const READS: Reads = Reads::Every;

    fn new(_: &'file Source) -> NoDyn {
        NoDyn::default()
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
