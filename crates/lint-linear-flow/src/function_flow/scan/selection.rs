use std::collections::BTreeSet;

use proc_macro2::{TokenStream, TokenTree};
use syn::parse::Parse;
use syn::visit::{self, Visit};
use syn::{Expr, ExprClosure, ExprMethodCall, ExprPath, Macro, PatIdent};

use lint_layout::Name;

use crate::finding::Finding;

#[derive(Debug, Default)]
pub struct Selection {
    bound: BTreeSet<Name>,
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
            .filter(|selection| selection.bound.intersection(&selection.read).next().is_none())
            .map(|_| Finding::IgnoredElement)
    }

    fn closure(predicate: &Expr) -> Option<&ExprClosure> {
        match predicate {
            Expr::Closure(closure) => Some(closure),
            _ => None,
        }
    }

    fn of_closure(closure: &ExprClosure) -> Selection {
        let mut selection = Selection::default();
        closure.inputs.iter().for_each(|input| selection.visit_pat(input));
        selection.visit_expr(&closure.body);
        selection
    }

    fn tokens_read(tokens: TokenStream) -> Vec<Name> {
        tokens.into_iter().flat_map(Self::tree_read).collect()
    }

    fn tree_read(tree: TokenTree) -> Vec<Name> {
        match tree {
            TokenTree::Ident(ident) => vec![Name::from(&ident)],
            TokenTree::Group(group) => Self::tokens_read(group.stream()),
            TokenTree::Punct(_) | TokenTree::Literal(_) => Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for Selection {
    fn visit_pat_ident(&mut self, binding: &'ast PatIdent) {
        self.bound.insert(Name::from(&binding.ident));
        visit::visit_pat_ident(self, binding);
    }

    fn visit_expr_path(&mut self, path: &'ast ExprPath) {
        self.read.extend(path.path.get_ident().map(Name::from));
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.read.extend(invocation.parse_body_with(TokenStream::parse).map(Self::tokens_read).unwrap_or_default());
    }
}
