use syntax::{Group, TokenStream, TokenTree};

use crate::syntax::Syntax;

#[derive(Debug)]
pub struct Transcriber;

struct Metavariable;

struct Copied;

impl Transcriber {
    const DOLLAR: char = '$';

    #[must_use]
    pub fn substituted(body: TokenStream) -> TokenStream {
        Self::trees(&body.into_iter().collect::<Vec<TokenTree>>()).into_iter().collect()
    }

    fn trees(trees: &[TokenTree]) -> Vec<TokenTree> {
        match trees.split_first() {
            None => Vec::new(),
            Some((first, rest)) => Self::stepped(first, rest),
        }
    }

    fn stepped(first: &TokenTree, rest: &[TokenTree]) -> Vec<TokenTree> {
        Syntax::punct(first)
            .filter(|punct| punct.as_char() == Self::DOLLAR)
            .and_then(|_| rest.split_first())
            .and_then(|(next, after)| Metavariable::replaced(next, after))
            .unwrap_or_else(|| Self::then(vec![Copied::kept(first)], rest))
    }

    fn then(head: Vec<TokenTree>, rest: &[TokenTree]) -> Vec<TokenTree> {
        head.into_iter().chain(Self::trees(rest)).collect()
    }
}

impl Metavariable {
    const REPEATS: [char; 3] = ['*', '+', '?'];
    const SEPARATED: usize = 2;

    fn replaced(next: &TokenTree, after: &[TokenTree]) -> Option<Vec<TokenTree>> {
        match next {
            TokenTree::Ident(name) => Some(Transcriber::then(vec![TokenTree::Ident(name.clone())], after)),
            TokenTree::Group(repeated) => Some(Transcriber::then(Copied::inner(repeated), Self::unrepeated(after))),
            TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        }
    }

    fn unrepeated(after: &[TokenTree]) -> &[TokenTree] {
        after
            .iter()
            .take(Self::SEPARATED)
            .position(|tree| Syntax::punct(tree).is_some_and(|punct| Self::REPEATS.contains(&punct.as_char())))
            .and_then(|at| after.get(at + 1..))
            .unwrap_or(after)
    }
}

impl Copied {
    fn kept(tree: &TokenTree) -> TokenTree {
        Syntax::group(tree).map_or_else(
            || tree.clone(),
            |group| TokenTree::Group(Group::new(group.delimiter(), Self::inner(group).into_iter().collect())),
        )
    }

    fn inner(group: &Group) -> Vec<TokenTree> {
        Transcriber::trees(&group.stream().into_iter().collect::<Vec<TokenTree>>())
    }
}
