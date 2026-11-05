use std::collections::BTreeSet;
use std::convert::Infallible;
use std::mem;
use std::str::FromStr;

use proc_macro2::{Literal, TokenStream, TokenTree};
use syn::parse::Parse;
use syn::visit::{self, Visit};
use syn::{
    Arm, Block, Expr, ExprClosure, ExprForLoop, ExprIf, ExprLet, ExprMethodCall, ExprPath, LitStr, Macro, Pat,
    PatIdent, Stmt,
};

use lint_layout::Name;

use crate::finding::Finding;

#[derive(Debug, Default)]
struct Bindings(BTreeSet<Name>);

impl Bindings {
    fn within<'pat>(pats: impl IntoIterator<Item = &'pat Pat>) -> BTreeSet<Name> {
        let mut bindings = Bindings::default();
        pats.into_iter().for_each(|pat| bindings.visit_pat(pat));
        bindings.0
    }

    fn declared(stmt: &Stmt) -> BTreeSet<Name> {
        match stmt {
            Stmt::Local(local) => Self::within([&local.pat]),
            _ => BTreeSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for Bindings {
    fn visit_pat_ident(&mut self, binding: &'ast PatIdent) {
        self.0.insert(Name::from(&binding.ident));
        visit::visit_pat_ident(self, binding);
    }
}

#[derive(Debug)]
struct Placeholders(Vec<Name>);

#[derive(Debug)]
struct Leading(Name);

impl FromStr for Leading {
    type Err = Infallible;

    fn from_str(piece: &str) -> Result<Leading, Infallible> {
        let word: String = piece
            .chars()
            .map_while(|ch| Some(ch).filter(|ch| ch.is_alphanumeric()).or(Some(ch).filter(|ch| *ch == '_')))
            .collect();
        Ok(Leading(Name::from(word.as_str())))
    }
}

impl FromStr for Placeholders {
    type Err = Infallible;

    fn from_str(text: &str) -> Result<Placeholders, Infallible> {
        Ok(Placeholders(
            text.match_indices('{')
                .filter_map(|(at, _)| text.get(at + 1..))
                .filter_map(|piece| piece.parse::<Leading>().ok())
                .map(|leading| leading.0)
                .collect(),
        ))
    }
}

#[derive(Debug)]
struct Mentions(Vec<Name>);

impl Mentions {
    fn of_tree(tree: TokenTree) -> Vec<Name> {
        match tree {
            TokenTree::Ident(ident) => vec![Name::from(&ident)],
            TokenTree::Group(group) => Mentions::from(group.stream()).0,
            TokenTree::Literal(literal) => Self::inside(literal),
            TokenTree::Punct(_) => Vec::new(),
        }
    }

    fn inside(literal: Literal) -> Vec<Name> {
        syn::parse2::<LitStr>(TokenStream::from(TokenTree::Literal(literal)))
            .ok()
            .and_then(|text| text.value().parse::<Placeholders>().ok())
            .map(|placeholders| placeholders.0)
            .unwrap_or_default()
    }
}

impl From<TokenStream> for Mentions {
    fn from(tokens: TokenStream) -> Mentions {
        Mentions(tokens.into_iter().flat_map(Self::of_tree).collect())
    }
}

#[derive(Debug)]
pub struct Selection {
    element: BTreeSet<Name>,
    read: BTreeSet<Name>,
}

impl Selection {
    const SELECTING: [&'static str; 4] = ["filter", "any", "all", "position"];

    pub fn ignoring(call: &ExprMethodCall) -> Option<Finding> {
        Some(call)
            .filter(|call| Self::SELECTING.iter().any(|selecting| call.method == selecting))
            .and_then(|call| call.args.first())
            .and_then(Self::closure)
            .map(Self::of_closure)
            .filter(|selection| selection.read.is_empty())
            .map(|_| Finding::IgnoredElement)
    }

    fn closure(predicate: &Expr) -> Option<&ExprClosure> {
        match predicate {
            Expr::Closure(closure) => Some(closure),
            _ => None,
        }
    }

    fn of_closure(closure: &ExprClosure) -> Selection {
        let mut selection = Selection { element: Bindings::within(&closure.inputs), read: BTreeSet::new() };
        selection.visit_expr(&closure.body);
        selection
    }

    fn hide(&mut self, hidden: &BTreeSet<Name>) -> BTreeSet<Name> {
        let (covered, visible) = mem::take(&mut self.element).into_iter().partition(|name| hidden.contains(name));
        self.element = visible;
        covered
    }

    fn if_let(&mut self, binding: &ExprLet, branch: &ExprIf) {
        self.visit_expr(&binding.expr);
        let covered = self.hide(&Bindings::within([&*binding.pat]));
        self.visit_block(&branch.then_branch);
        self.element.extend(covered);
        branch.else_branch.iter().for_each(|(_, otherwise)| self.visit_expr(otherwise));
    }
}

impl<'ast> Visit<'ast> for Selection {
    fn visit_expr_closure(&mut self, closure: &'ast ExprClosure) {
        let covered = self.hide(&Bindings::within(&closure.inputs));
        self.visit_expr(&closure.body);
        self.element.extend(covered);
    }

    fn visit_arm(&mut self, arm: &'ast Arm) {
        let covered = self.hide(&Bindings::within([&arm.pat]));
        arm.guard.iter().for_each(|(_, guard)| self.visit_expr(guard));
        self.visit_expr(&arm.body);
        self.element.extend(covered);
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let covered: Vec<BTreeSet<Name>> = block
            .stmts
            .iter()
            .map(|stmt| {
                self.visit_stmt(stmt);
                self.hide(&Bindings::declared(stmt))
            })
            .collect();
        self.element.extend(covered.into_iter().flatten());
    }

    fn visit_expr_for_loop(&mut self, looping: &'ast ExprForLoop) {
        self.visit_expr(&looping.expr);
        let covered = self.hide(&Bindings::within([&*looping.pat]));
        self.visit_block(&looping.body);
        self.element.extend(covered);
    }

    fn visit_expr_if(&mut self, branch: &'ast ExprIf) {
        match &*branch.cond {
            Expr::Let(binding) => self.if_let(binding, branch),
            _ => visit::visit_expr_if(self, branch),
        }
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        let mentioned = path.path.get_ident().map(Name::from).filter(|name| self.element.contains(name));
        self.read.extend(mentioned);
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let mentions = invocation.parse_body_with(TokenStream::parse).map(Mentions::from).map(|mentions| mentions.0);
        self.read.extend(mentions.unwrap_or_default().into_iter().filter(|name| self.element.contains(name)));
    }
}
