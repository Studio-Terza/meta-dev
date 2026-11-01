use proc_macro2::{Ident, Literal, TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::{Attribute, Lit, Macro};

use lint_layout::{LineNumber, Name};

use crate::feature_word::FeatureWord;
use crate::finding::Finding;
use crate::flagged::Flagged;

#[derive(Debug, Default)]
pub(crate) struct TestFeatures {
    found: Vec<Flagged>,
}

impl TestFeatures {
    const CONDITIONS: [&str; 2] = ["cfg", "cfg_attr"];
    const FEATURE: &str = "feature";
    const VALUE_INDEX: usize = 2;

    pub(crate) fn in_file(file: &syn::File) -> Vec<Flagged> {
        let mut features = TestFeatures::default();
        features.visit_file(file);
        features.found
    }

    fn is_condition(path: &syn::Path) -> bool {
        Self::CONDITIONS.iter().any(|condition| path.is_ident(condition))
    }

    fn flagged(key: &Ident, value: &Literal) -> Option<Flagged> {
        match Lit::new(value.clone()) {
            Lit::Str(feature) if key == Self::FEATURE => {
                let name = Name::from(feature.value().as_str());
                FeatureWord::is_test(&name).then(|| Flagged::new(LineNumber::from(key.span()), Finding::Feature(name)))
            }
            _ => None,
        }
    }

    fn scanned(tokens: TokenStream) -> Vec<Flagged> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let here = trees.iter().zip(trees.iter().skip(Self::VALUE_INDEX)).filter_map(|pair| match pair {
            (TokenTree::Ident(key), TokenTree::Literal(value)) => Self::flagged(key, value),
            _ => None,
        });
        let nested = trees.iter().filter_map(|tree| match tree {
            TokenTree::Group(group) => Some(group.stream()),
            _ => None,
        });
        here.chain(nested.flat_map(Self::scanned)).collect()
    }
}

impl<'ast> Visit<'ast> for TestFeatures {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.found.extend(
            Some(attribute)
                .filter(|attribute| Self::is_condition(attribute.path()))
                .into_iter()
                .flat_map(|attribute| Self::scanned(attribute.to_token_stream())),
        );
        visit::visit_attribute(self, attribute);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.found.extend(
            Some(invocation)
                .filter(|invocation| Self::is_condition(&invocation.path))
                .into_iter()
                .flat_map(|invocation| Self::scanned(invocation.tokens.clone())),
        );
    }
}
