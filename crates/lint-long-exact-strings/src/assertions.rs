mod bindings;
mod compared;
mod finding;
mod pinning;
mod table_rows;

use std::path::Path;

use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ImplItemFn, ItemFn, Macro, Token};

use lint_core::{MacroAliases, MacroCalls, Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name, Word, words};

use bindings::Bindings;
use compared::Compared;
use finding::Finding;
use table_rows::TableRows;

pub struct Assertions<'file> {
    path: &'file Path,
    constants: Bindings,
    locals: Bindings,
    tables: MacroAliases,
    found: Vec<Violation>,
}

impl Assertions<'_> {
    const EQUALITIES: [Word<'static>; 2] = words!["assert_eq", "assert_ne"];
    const ASSERT: &str = "assert";
    const TABLE: Word<'static> = Word::new("table");

    pub fn violations(file: &SourceFile) -> Vec<Violation> {
        let mut assertions = Assertions {
            path: file.path(),
            constants: Bindings::constants(file.syntax()),
            locals: Bindings::default(),
            tables: MacroAliases::in_file(file.syntax(), Self::TABLE),
            found: Vec::new(),
        };
        assertions.visit_file(file.syntax());
        assertions.found
    }

    fn compared(&self, invocation: &Macro, name: &Name) -> Option<Vec<Expr>> {
        let arguments = || {
            invocation
                .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
                .map(|arguments| arguments.into_iter().collect::<Vec<Expr>>())
                .unwrap_or_default()
        };
        match name {
            _ if name.matches::<lint_layout::Whole>(&Self::EQUALITIES) => {
                Some(arguments().into_iter().take(2).collect())
            }
            _ if *name == Self::ASSERT => Some(arguments().into_iter().take(1).flat_map(Self::sides).collect()),
            _ if self.tables.call(name) => Some(TableRows::expected(&invocation.tokens)),
            _ => None,
        }
    }

    fn sides(condition: Expr) -> Vec<Expr> {
        match condition {
            Expr::Binary(binary) if matches!(binary.op, BinOp::Eq(_) | BinOp::Ne(_)) => {
                vec![*binary.left, *binary.right]
            }
            _ => Vec::new(),
        }
    }

    fn judged(&mut self, invocation: &Macro, compared: &[Expr]) {
        let line =
            invocation.path.segments.first().map_or(LineNumber::FIRST, |first| LineNumber::from(first.ident.span()));
        let judge = Compared { constants: &self.constants, locals: &self.locals };
        let pinned = compared.iter().find_map(|side| judge.pinning(side));
        self.found.extend(pinned.map(|pinning| Violation::new(Site::new(self.path, line), Finding::from(pinning))));
    }

    fn within_body(&mut self, locals: Bindings, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.locals, locals);
        visit(self);
        self.locals = outer;
    }
}

impl<'ast> Visit<'ast> for Assertions<'_> {
    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.within_body(Bindings::locals(&item.block), |assertions| visit::visit_item_fn(assertions, item));
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.within_body(Bindings::locals(&item.block), |assertions| visit::visit_impl_item_fn(assertions, item));
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        let name = invocation.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default();
        match self.compared(invocation, &name) {
            Some(compared) => self.judged(invocation, &compared),
            None => MacroCalls::within(invocation).iter().for_each(|inner| self.visit_macro(inner)),
        }
    }
}
