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
        body.into_iter().fold(Unvarying::default(), Unvarying::read).ended().into_iter().collect()
    }

    fn dollar(tree: &TokenTree) -> Option<()> {
        match tree {
            TokenTree::Punct(mark) => (mark.as_char() == Expansion::DOLLAR).then_some(()),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }

    fn repeats(tree: &TokenTree) -> Option<()> {
        match tree {
            TokenTree::Punct(mark) => Expansion::REPEATS.contains(&mark.as_char()).then_some(()),
            TokenTree::Group(_) | TokenTree::Ident(_) | TokenTree::Literal(_) => None,
        }
    }
}

#[derive(Debug, Default)]
struct Unvarying {
    kept: Vec<TokenTree>,
    mode: Mode,
}

#[derive(Debug, Default)]
enum Mode {
    #[default]
    Plain,
    Dollar,
    Repeated(Option<TokenTree>),
}

impl Unvarying {
    fn read(self, next: TokenTree) -> Unvarying {
        let Unvarying { kept, mode } = self;
        match (mode, next) {
            (Mode::Dollar, TokenTree::Group(group)) => Unvarying {
                kept: kept.into_iter().chain(Expansion::unvaried(group.stream())).collect(),
                mode: Mode::Repeated(None),
            },
            (Mode::Repeated(_), tree) if Expansion::repeats(&tree).is_some() => Unvarying { kept, mode: Mode::Plain },
            (Mode::Repeated(None), tree) => Unvarying { kept, mode: Mode::Repeated(Some(tree)) },
            (Mode::Repeated(Some(held)), tree) => Unvarying { kept, mode: Mode::Plain }.flushed(Some(held)).read(tree),
            (Mode::Plain | Mode::Dollar, tree) if Expansion::dollar(&tree).is_some() => {
                Unvarying { kept, mode: Mode::Dollar }
            }
            (Mode::Plain, TokenTree::Group(group)) => {
                let inner = TokenTree::Group(Group::new(group.delimiter(), Expansion::unvaried(group.stream())));
                Unvarying { kept: kept.into_iter().chain([inner]).collect(), mode: Mode::Plain }
            }
            (Mode::Plain | Mode::Dollar, tree) => {
                Unvarying { kept: kept.into_iter().chain([tree]).collect(), mode: Mode::Plain }
            }
        }
    }

    fn flushed(self, held: Option<TokenTree>) -> Unvarying {
        held.into_iter().fold(self, Unvarying::read)
    }

    fn ended(self) -> Vec<TokenTree> {
        match self.mode {
            Mode::Repeated(held) => Unvarying { kept: self.kept, mode: Mode::Plain }.flushed(held).kept,
            Mode::Plain | Mode::Dollar => self.kept,
        }
    }
}
