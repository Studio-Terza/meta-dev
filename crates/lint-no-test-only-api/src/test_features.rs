use std::borrow::Cow;
use std::iter;

use proc_macro2::{Literal, TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::{Attribute, Lit, Macro};

use lint_layout::LineNumber;
use lint_layout::Name;

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

    fn condition(path: &syn::Path) -> Option<&syn::Path> {
        Self::CONDITIONS.iter().find(|condition| path.is_ident(condition)).map(|_| path)
    }

    fn feature_key(tree: &TokenTree) -> Option<LineNumber> {
        match tree {
            TokenTree::Ident(key) if key == Self::FEATURE => Some(LineNumber::from(key.span())),
            _ => None,
        }
    }

    fn flagged(line: LineNumber, value: Literal) -> Option<Flagged> {
        match Lit::new(value) {
            Lit::Str(feature) => {
                let name = Name::from(feature.value().as_str());
                FeatureWord::test_word(&name).map(|_| Flagged::new(line, Finding::Feature(Cow::Owned(name))))
            }
            _ => None,
        }
    }

    fn scanned(tokens: TokenStream) -> Vec<Flagged> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let keys: Vec<Option<LineNumber>> =
            iter::repeat_n(None, Self::VALUE_INDEX).chain(trees.iter().map(Self::feature_key)).collect();
        let nested: Vec<Flagged> = trees
            .iter()
            .filter_map(|tree| match tree {
                TokenTree::Group(group) => Some(group.stream()),
                _ => None,
            })
            .flat_map(Self::scanned)
            .collect();
        let mut here: Vec<Flagged> = trees
            .into_iter()
            .zip(keys)
            .filter_map(|pair| match pair {
                (TokenTree::Literal(value), Some(line)) => Self::flagged(line, value),
                _ => None,
            })
            .collect();
        here.extend(nested);
        here
    }
}

impl<'ast> Visit<'ast> for TestFeatures {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.found.extend(
            Self::condition(attribute.path()).into_iter().flat_map(|_| Self::scanned(attribute.to_token_stream())),
        );
        visit::visit_attribute(self, attribute);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.found.extend(
            Self::condition(&invocation.path).into_iter().flat_map(|_| Self::scanned(invocation.tokens.clone())),
        );
    }
}
