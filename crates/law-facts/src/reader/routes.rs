use std::collections::{BTreeMap, BTreeSet};

use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Expr, ExprPath, Macro, Token};

use lint_layout::Name;

#[derive(Debug, Default)]
pub(crate) struct Routes(BTreeMap<Name, BTreeSet<Name>>);

impl Routes {
    pub(crate) fn read(invocation: &Macro) -> BTreeMap<Name, BTreeSet<Name>> {
        let arguments = invocation.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated);
        let mut routes = Routes::default();
        arguments.iter().flatten().for_each(|argument| routes.visit_expr(argument));
        routes.0
    }
}

impl<'ast> Visit<'ast> for Routes {
    fn visit_expr_path(&mut self, found: &'ast ExprPath) {
        let member = found.path.segments.last();
        let owner = found.path.segments.iter().rev().nth(1);
        member.zip(owner).into_iter().for_each(|(member, owner)| {
            self.0.entry(Name::from(&owner.ident)).or_default().insert(Name::from(&member.ident));
        });
        visit::visit_expr_path(self, found);
    }
}
