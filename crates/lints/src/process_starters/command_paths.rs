use std::collections::BTreeSet;

use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprMethodCall, Ident, ItemUse, Macro, Token, UseTree};

use lint_core::{LineNumber, SourceFile};

#[derive(Debug, Default)]
pub(crate) struct CommandPaths {
    bound: BTreeSet<Ident>,
    started: Vec<LineNumber>,
    open_starts: Vec<LineNumber>,
}

impl CommandPaths {
    const MODULE: &str = "process";
    const STARTER: &str = "Command";
    const BUILT: &str = "new";
    const STARTS: [&str; 3] = ["spawn", "output", "status"];

    pub(crate) fn found(file: &SourceFile) -> Vec<LineNumber> {
        let mut paths = CommandPaths::default();
        paths.visit_file(file.syntax());
        paths.started
    }

    fn imported(&mut self, tree: &UseTree, under_process: bool) {
        match tree {
            UseTree::Path(path) => self.imported(&path.tree, path.ident == Self::MODULE),
            UseTree::Name(name) if under_process && name.ident == Self::STARTER => {
                self.bound.insert(name.ident.clone());
            }
            UseTree::Rename(name) if under_process && name.ident == Self::STARTER => {
                self.bound.insert(name.rename.clone());
            }
            UseTree::Glob(glob) if under_process => {
                self.bound.insert(Ident::new(Self::STARTER, glob.star_token.span));
            }
            UseTree::Group(group) => group.items.iter().for_each(|item| self.imported(item, under_process)),
            UseTree::Name(_) | UseTree::Rename(_) | UseTree::Glob(_) => {}
        }
    }

    fn built(&self, call: &ExprCall) -> bool {
        match &*call.func {
            Expr::Path(path) => {
                let names: Vec<&Ident> = path.path.segments.iter().map(|segment| &segment.ident).collect();
                Self::spells_command(&names)
                    || matches!(names.as_slice(), [starter, built] if self.bound.contains(*starter) && *built == Self::BUILT)
            }
            _ => false,
        }
    }

    fn spells_command(names: &[&Ident]) -> bool {
        match names {
            [module, starter, built, ..]
                if *module == Self::MODULE && *starter == Self::STARTER && *built == Self::BUILT =>
            {
                true
            }
            [_, rest @ ..] => Self::spells_command(rest),
            [] => false,
        }
    }

    fn rooted(&self, receiver: &Expr) -> bool {
        match receiver {
            Expr::MethodCall(call) => self.rooted(&call.receiver),
            Expr::Call(call) => self.built(call),
            _ => false,
        }
    }
}

impl<'ast> Visit<'ast> for CommandPaths {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.imported(&item.tree, false);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        let started =
            (self.open_starts.is_empty() && self.built(call)).then(|| LineNumber::from(call.paren_token.span.open()));
        self.started.extend(started);
        visit::visit_expr_call(self, call);
    }

    fn visit_macro(&mut self, called: &'ast Macro) {
        let arguments = called.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated);
        arguments.iter().flatten().for_each(|argument| self.visit_expr(argument));
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let started = (Self::STARTS.iter().any(|start| call.method == start) && self.rooted(&call.receiver))
            .then(|| LineNumber::from(call.method.span()));
        let depth = self.open_starts.len();
        self.started.extend(started);
        self.open_starts.extend(started);
        visit::visit_expr_method_call(self, call);
        self.open_starts.truncate(depth);
    }
}
