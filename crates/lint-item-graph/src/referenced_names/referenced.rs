mod path_tokens;

use std::collections::BTreeMap;
use std::ptr;

use lint_core::Syntax;
use lint_layout::{Name, UsePath};
use proc_macro2::{TokenStream, TokenTree};

use super::ReferencedNames;
use super::parameters::Parameters;
use crate::scope::Scope;
use path_tokens::PathTokens;

#[derive(Debug)]
struct Segment<'token> {
    root: &'token TokenTree,
    name: Name,
}

#[derive(Debug, Default)]
pub struct Referenced {
    local: Vec<UsePath>,
    across: Vec<UsePath>,
}

impl Referenced {
    pub fn of_tokens(scope: &Scope<'_>, tokens: TokenStream) -> Self {
        let path = PathTokens::new(tokens);
        let idents: Vec<(&TokenTree, &TokenTree, Name)> = path
            .as_ref()
            .iter()
            .zip(path.roots())
            .filter_map(|(token, root)| {
                Syntax::token_ident(token).zip(root).map(|(ident, root)| (token, root, Name::from(ident)))
            })
            .collect();
        let local: Vec<Segment<'_>> = idents
            .iter()
            .filter_map(|(token, root, name)| scope.kept(token, root, name.clone()).map(|name| Segment { root, name }))
            .collect();
        let across: Vec<Segment<'_>> = idents
            .into_iter()
            .flat_map(|(token, root, name)| {
                scope.spelled(token, root, name).into_iter().map(move |name| Segment { root, name })
            })
            .collect();
        let nested = path.as_ref().iter().filter_map(Syntax::group).map(|group| Self::of_tokens(scope, group.stream()));
        let own = Self { local: Self::paths(&local), across: Self::paths(&across) };
        nested.fold(own, Self::merged)
    }

    fn merged(mut self, other: Self) -> Self {
        self.local.extend(other.local);
        self.across.extend(other.across);
        self
    }

    pub fn owned(self, owner: Name, parameters: &Parameters) -> ReferencedNames {
        ReferencedNames {
            declared: BTreeMap::from([(owner.clone(), parameters.unbound(self.local))]),
            across: BTreeMap::from([(owner, parameters.unbound(self.across))]),
            ..ReferencedNames::default()
        }
    }

    fn paths(segments: &[Segment<'_>]) -> Vec<UsePath> {
        segments
            .chunk_by(|left, right| ptr::eq(left.root, right.root))
            .map(|path| path.iter().map(|segment| segment.name.clone()).collect())
            .collect()
    }
}
