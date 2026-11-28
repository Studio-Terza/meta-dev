use std::collections::BTreeSet;

use invariants::Ignoring;
use proc_macro2::{Group, TokenStream, TokenTree};
use syn::parse::discouraged::Speculative;
use syn::parse::{ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Block, Expr, Macro, Stmt, Token};

use crate::ignored::Ignored;

#[derive(Debug)]
pub(crate) struct Expansion(Vec<Stmt>);

impl Expansion {
    const DOLLAR: char = '$';
    const REPEATS: [char; 3] = ['*', '+', '?'];

    pub(crate) fn of(invocation: &Macro) -> Expansion {
        let bodies = Expansion::defining(invocation)
            .map_or_else(|| vec![invocation.tokens.clone()], |()| Expansion::transcribers(invocation.tokens.clone()));
        Expansion(bodies.into_iter().flat_map(Expansion::statements).collect())
    }

    pub(crate) fn substituted(invocation: &Macro) -> Expansion {
        let bodies =
            Expansion::defining(invocation).map_or_default(|()| Expansion::transcribers(invocation.tokens.clone()));
        Expansion(bodies.into_iter().map(Expansion::unvaried).flat_map(Expansion::statements).collect())
    }

    pub(crate) fn defining(invocation: &Macro) -> Option<()> {
        invocation.path.is_ident("macro_rules").then_some(())
    }

    pub(crate) fn walk<Visitor: for<'ast> Visit<'ast>>(&self, visitor: &mut Visitor) {
        self.0.iter().for_each(|statement| visitor.visit_stmt(statement));
    }

    fn transcribers(rules: TokenStream) -> Vec<TokenStream> {
        let groups = rules.into_iter().filter_map(|tree| match tree {
            TokenTree::Group(group) => Some(group.stream()),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        });
        groups.skip(1).step_by(2).collect()
    }

    fn statements(tokens: TokenStream) -> Vec<Stmt> {
        Expansion::parsed.parse2(tokens).ignoring(Ignored::Unparsed).unwrap_or_default()
    }

    fn parsed(input: ParseStream) -> syn::Result<Vec<Stmt>> {
        let attempt = input.fork();
        Block::parse_within(&attempt).ignoring(Ignored::Unparsed).map_or_else(
            || {
                let arguments = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
                Ok(arguments.into_iter().map(|argument| Stmt::Expr(argument, None)).collect())
            },
            |statements| {
                input.advance_to(&attempt);
                Ok(statements)
            },
        )
    }

    fn unvaried(body: TokenStream) -> TokenStream {
        let trees: Vec<TokenTree> = body.into_iter().collect();
        let dropped = Expansion::dropped(&trees);
        let kept = trees.iter().enumerate().filter(|(at, _)| !dropped.contains(at));
        kept.flat_map(|(at, tree)| Expansion::kept(tree, at.checked_sub(1).and_then(|before| trees.get(before))))
            .collect()
    }

    fn kept(tree: &TokenTree, before: Option<&TokenTree>) -> Vec<TokenTree> {
        let repeated = before.and_then(Expansion::dollar);
        match tree {
            TokenTree::Group(group) => repeated.map_or_else(
                || vec![TokenTree::Group(Group::new(group.delimiter(), Expansion::unvaried(group.stream())))],
                |()| Expansion::unvaried(group.stream()).into_iter().collect(),
            ),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => vec![tree.clone()],
        }
    }

    fn dropped(trees: &[TokenTree]) -> BTreeSet<usize> {
        let dollars = trees.iter().enumerate().filter_map(|(at, tree)| Expansion::dollar(tree).map(|()| at));
        dollars
            .flat_map(|at| {
                let repeated = trees.get(at + 1).and_then(|next| match next {
                    TokenTree::Group(_) => Expansion::repeat_end(trees, at + 2),
                    TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
                });
                [at].into_iter().chain(repeated.map_or_default(|end| (at + 2..=end).collect::<Vec<usize>>()))
            })
            .collect()
    }

    fn repeat_end(trees: &[TokenTree], start: usize) -> Option<usize> {
        [start, start + 1].into_iter().find(|at| {
            trees.get(*at).is_some_and(|tree| match tree {
                TokenTree::Punct(mark) => Expansion::REPEATS.contains(&mark.as_char()),
                TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_) => false,
            })
        })
    }

    fn dollar(tree: &TokenTree) -> Option<()> {
        match tree {
            TokenTree::Punct(mark) => (mark.as_char() == Expansion::DOLLAR).then_some(()),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }
}
