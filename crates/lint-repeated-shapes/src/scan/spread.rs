use proc_macro2::{Span, TokenStream};

#[derive(Debug)]
pub struct Spread;

impl Spread {
    pub fn joined(tokens: &TokenStream) -> Span {
        let spans: Vec<Span> = tokens.clone().into_iter().map(|tree| tree.span()).collect();
        let first = spans.first().copied().unwrap_or_else(Span::call_site);
        spans.last().and_then(|last| first.join(*last)).unwrap_or(first)
    }
}
