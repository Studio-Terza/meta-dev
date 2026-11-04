mod path_tokens;

use std::collections::BTreeMap;
use std::ptr;

use lint_layout::{Name, UsePath};
use proc_macro2::{TokenStream, TokenTree};

use super::ReferencedNames;
use super::parameters::Parameters;
use crate::scope::{Reading, Scope};
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
    pub fn of_tokens(scope: &Scope<'_>, tokens: TokenStream) -> Referenced {
        let path = PathTokens::new(tokens);
        let local: Vec<Segment<'_>> =
            path.tokens().iter().zip(path.roots()).filter_map(|(token, root)| Self::kept(scope, token, root)).collect();
        let across = match scope.reads() {
            Reading::Declarations => Vec::new(),
            Reading::Bodies => Self::spelled(scope, &path),
        };
        let nested = path.tokens().iter().filter_map(|token| match token {
            TokenTree::Group(group) => Some(Referenced::of_tokens(scope, group.stream())),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        });
        let own = Referenced { local: Self::paths(&local), across: Self::paths(&across) };
        nested.fold(own, Referenced::merged)
    }

    fn kept<'token>(
        scope: &Scope<'_>,
        token: &'token TokenTree,
        root: Option<&'token TokenTree>,
    ) -> Option<Segment<'token>> {
        match (token, root) {
            (TokenTree::Ident(ident), Some(root)) => {
                scope.kept(token, root, Name::from(ident)).map(|name| Segment { root, name })
            }
            _ => None,
        }
    }

    fn spelled<'token>(scope: &Scope<'_>, path: &'token PathTokens) -> Vec<Segment<'token>> {
        path.tokens()
            .iter()
            .zip(path.roots())
            .filter_map(|(token, root)| match (token, root) {
                (TokenTree::Ident(ident), Some(root)) => Some((token, root, Name::from(ident))),
                _ => None,
            })
            .flat_map(|(token, root, name)| {
                scope.spelled(token, root, name).into_iter().map(move |name| Segment { root, name })
            })
            .collect()
    }

    fn merged(mut self, other: Referenced) -> Referenced {
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
