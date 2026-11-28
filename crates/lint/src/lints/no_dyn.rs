use proc_macro2::{TokenStream, TokenTree};
use syn::{Macro, TypeTraitObject};

use crate::finding::{Findings, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Lint, Reads};
use crate::source::Source;

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

impl<'file> Lint<'file> for NoDyn {
    const HEADLINE: &'static str = "no dyn: every type is known at compile time, so code takes a generic, returns impl Trait or names an enum instead of a trait object";
    const READS: Reads = Reads::Every;

    type Fact = Findings;

    fn new(_: &'file Source) -> NoDyn {
        NoDyn::default()
    }

    fn fact(self, source: &'file Source) -> Findings {
        Findings::found(NoDyn::HEADLINE, source.shown(), self.found)
    }
}
