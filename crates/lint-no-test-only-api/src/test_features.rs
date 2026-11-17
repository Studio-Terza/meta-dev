use std::borrow::Cow;
use std::iter;

use syntax::ToTokens;
use syntax::{Attribute, Lit, Macro};
use syntax::{Group, Literal, TokenStream, TokenTree};

use lint_core::Syntax;
use lint_layout::LineNumber;
use lint_layout::Name;
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use std::path::Path;

use lint_core::Violation;

use crate::feature_word::FeatureWord;
use crate::finding::Finding;
use crate::flagged::Flagged;

#[derive(Debug)]
pub(crate) struct TestFeatures<'file> {
    path: &'file Path,
}

impl TestFeatures<'_> {
    const CONDITIONS: [&str; 2] = ["cfg", "cfg_attr"];
    const FEATURE: &str = "feature";
    const VALUE_INDEX: usize = 2;

    fn condition(path: &syntax::Path) -> Option<&syntax::Path> {
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

impl<'file> Fold for TestFeatures<'file> {
    type Found = Vec<Violation<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::Attribute, Kind::Macro]
    }

    fn on_attribute(&self, found: Self::Found, attribute: &Attribute, _: &Facts<'_, Attribute>) -> Visit<Self::Found> {
        Visit::adding(
            found,
            Self::condition(attribute.path())
                .into_iter()
                .flat_map(|_| Self::scanned(attribute.to_token_stream()))
                .map(|flagged| flagged.in_file(self.path)),
            Descend::Into,
        )
    }

    fn on_macro(&self, found: Self::Found, invocation: &Macro, _: &Facts<'_, Macro>) -> Visit<Self::Found> {
        Visit::adding(
            found,
            Self::condition(&invocation.path)
                .into_iter()
                .flat_map(|_| Self::scanned(invocation.tokens.clone()))
                .map(|flagged| flagged.in_file(self.path)),
            Descend::Over,
        )
    }
}

impl<'file> From<&'file Path> for TestFeatures<'file> {
    fn from(path: &'file Path) -> TestFeatures<'file> {
        TestFeatures { path }
    }
}
