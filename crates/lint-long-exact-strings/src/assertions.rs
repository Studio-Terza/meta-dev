mod bindings;
mod compared;
mod finding;
mod pinning;
mod table_rows;

use std::path::Path;

use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ImplItemFn, ItemFn, Macro};

use lint_core::{MacroAliases, MacroCalls, Macros, Site, SourceFile, Violation};
use lint_layout::{LineNumber, Name, Whole, Word, words};

use bindings::Bindings;
use compared::Compared;
use finding::Finding;
use table_rows::TableRows;

#[derive(Debug)]
pub struct Assertions<'file> {
    path: &'file Path,
    constants: Bindings<'file>,
    locals: Bindings<'file>,
    tables: MacroAliases,
    found: Vec<Violation<'file>>,
}

impl<'file> From<&'file SourceFile> for Assertions<'file> {
    fn from(file: &'file SourceFile) -> Assertions<'file> {
        let mut assertions = Assertions {
            path: file.path(),
            constants: Bindings::constants(file.syntax()),
            locals: Bindings::default(),
            tables: MacroAliases::in_file(file.syntax(), Self::TABLE),
            found: Vec::new(),
        };
        assertions.visit_file(file.syntax());
        assertions
    }
}

impl<'file> From<Assertions<'file>> for Vec<Violation<'file>> {
    fn from(assertions: Assertions<'file>) -> Vec<Violation<'file>> {
        assertions.found
    }
}

impl<'file> Assertions<'file> {
    const TABLE: Word<'static> = Word::new("table");

    fn judged(&mut self, invocation: &Macro, compared: &[Expr]) {
        let line =
            invocation.path.segments.first().map_or(LineNumber::FIRST, |first| LineNumber::from(first.ident.span()));
        let judge = Compared::new(&self.constants, &self.locals);
        let pinned = compared.iter().find_map(|side| judge.pinning(side));
        self.found.extend(pinned.map(|pinning| Violation::new(Site::new(self.path, line), Finding::from(pinning))));
    }

    fn within_body(&mut self, locals: Bindings<'file>, visit: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.locals, locals);
        visit(self);
        self.locals = outer;
    }
}

impl<'file> Visit<'file> for Assertions<'file> {
    fn visit_item_fn(&mut self, item: &'file ItemFn) {
        self.within_body(Bindings::locals(&item.block), |assertions| visit::visit_item_fn(assertions, item));
    }

    fn visit_impl_item_fn(&mut self, item: &'file ImplItemFn) {
        self.within_body(Bindings::locals(&item.block), |assertions| visit::visit_impl_item_fn(assertions, item));
    }

    fn visit_macro(&mut self, invocation: &'file Macro) {
        self.met(invocation);
    }
}

impl Assertions<'_> {
    fn met(&mut self, invocation: &Macro) {
        match Assertion::named(
            &invocation.path.segments.last().map(|last| Name::from(&last.ident)).unwrap_or_default(),
            &self.tables,
        )
        .compared(invocation)
        {
            Some(compared) => self.judged(invocation, &compared),
            None => MacroCalls::within(invocation).iter().for_each(|inner| self.met(inner)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assertion {
    Equality,
    Bare,
    Table,
    Other,
}

impl Assertion {
    const EQUALITIES: [Word<'static>; 2] = words!["assert_eq", "assert_ne"];
    const ASSERT: &str = "assert";
    const SIDES_LENGTH: usize = 2;

    fn named(name: &Name, tables: &MacroAliases) -> Assertion {
        name.matches::<Whole>(&Self::EQUALITIES)
            .map(|_| Assertion::Equality)
            .or_else(|| (*name == Self::ASSERT).then_some(Assertion::Bare))
            .or_else(|| tables.call(name).map(|_| Assertion::Table))
            .unwrap_or(Assertion::Other)
    }

    fn compared(self, invocation: &Macro) -> Option<Vec<Expr>> {
        match self {
            Assertion::Equality => Some(Macros::arguments(invocation).into_iter().take(Self::SIDES_LENGTH).collect()),
            Assertion::Bare => Some(Macros::arguments(invocation).into_iter().take(1).flat_map(Self::sides).collect()),
            Assertion::Table => Some(TableRows::expected(&invocation.tokens)),
            Assertion::Other => None,
        }
    }

    fn sides(condition: Expr) -> Vec<Expr> {
        match condition {
            Expr::Binary(binary) => Some(binary)
                .filter(|binary| matches!(binary.op, BinOp::Eq(_) | BinOp::Ne(_)))
                .map(|binary| vec![*binary.left, *binary.right])
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }
}
