use std::borrow::Cow;
use std::iter;

use proc_macro2::{Group, Literal, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Attribute, Lit, Macro};

use lint_core::Syntax;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Finds, Kind, Watch};

use std::path::Path;

use lint_core::Violation;

use crate::feature_word::FeatureWord;
use crate::finding::Finding;
use crate::flagged::Flagged;

#[derive(Debug)]
pub(crate) struct TestFeatures<'file> {
    path: &'file Path,
    found: Vec<Flagged>,
}

impl<'file> TestFeatures<'file> {
    const CONDITIONS: [&str; 2] = ["cfg", "cfg_attr"];
    const FEATURE: &str = "feature";
    const VALUE_INDEX: usize = 2;

    pub(crate) fn new(path: &'file Path) -> TestFeatures<'file> {
        TestFeatures { path, found: Vec::new() }
    }

    fn condition(path: &syn::Path) -> Option<&syn::Path> {
        Self::CONDITIONS.iter().find(|condition| path.is_ident(condition)).map(|_| path)
    }

    fn feature_key(tree: &TokenTree) -> Option<LineNumber> {
        Syntax::token_ident(tree).filter(|key| *key == Self::FEATURE).map(|key| LineNumber::from(key.span()))
    }

    fn flagged(line: LineNumber, value: Literal) -> Option<Flagged> {
        match Lit::new(value) {
            Lit::Str(feature) => Some(Name::from(feature.value().as_str()))
                .filter(|name| name.matches::<FeatureWord>(&FeatureWord::TEST_WORDS).is_some())
                .map(|name| Flagged::new(line, Finding::Feature(Cow::Owned(name)))),
            _ => None,
        }
    }

    fn scanned(tokens: TokenStream) -> Vec<Flagged> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let keys: Vec<Option<LineNumber>> =
            iter::repeat_n(None, Self::VALUE_INDEX).chain(trees.iter().map(Self::feature_key)).collect();
        let nested: Vec<Flagged> =
            trees.iter().filter_map(Syntax::group).map(Group::stream).flat_map(Self::scanned).collect();
        trees
            .iter()
            .zip(keys)
            .filter_map(|(tree, key)| key.zip(Syntax::token_literal(tree).cloned()))
            .filter_map(|(line, value)| Self::flagged(line, value))
            .chain(nested)
            .collect()
    }
}

impl<'ast> Watch<'ast> for TestFeatures<'_> {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Attribute, Kind::Macro]
    }

    fn on_attribute(&mut self, attribute: &'ast Attribute) -> Descend {
        self.found.extend(
            Self::condition(attribute.path()).into_iter().flat_map(|_| Self::scanned(attribute.to_token_stream())),
        );
        Descend::Into
    }

    fn on_macro(&mut self, invocation: &'ast Macro) -> Descend {
        self.found.extend(
            Self::condition(&invocation.path).into_iter().flat_map(|_| Self::scanned(invocation.tokens.clone())),
        );
        Descend::Over
    }
}

impl<'file> Finds<'file> for TestFeatures<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let path = self.path;
        self.found.into_iter().map(|flagged| flagged.in_file(path)).collect()
    }
}
