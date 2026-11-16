mod bindings;
mod compared;
mod finding;
mod pinning;
mod table_rows;

use std::path::Path;

use lint_core::{Macros, SourceFile};

use syntax::Punctuated;
use syntax::{BinOp, Expr, ImplItemFn, ItemConst, ItemFn, ItemUse, Local, Macro, MacroTree, Token};

use lint_core::{MacroAliases, Site, Violation};
use lint_layout::{LineNumber, Name, Whole, Word, words};
use lint_sweep::{Descend, Facts, Finds, Kind, Watch};

use bindings::{Bindings, BodyId, Bound, Scopes};
use compared::Compared;
use finding::Finding;
use table_rows::TableRows;

#[derive(Debug)]
pub struct Assertions<'file> {
    file: &'file SourceFile,
    kinds: &'static [Kind],
    scopes: Scopes,
    tables: MacroAliases,
    met: Vec<Met>,
}

#[derive(Debug)]
struct Met {
    invocation: Macro,
    body: Option<BodyId>,
}

impl<'file> Assertions<'file> {
    const TABLE: Word<'static> = Word::new("table");
    const WATCHED: &'static [Kind] =
        &[Kind::ItemFn, Kind::ImplItemFn, Kind::ItemConst, Kind::Local, Kind::ItemUse, Kind::Macro];

    pub fn new(file: &'file SourceFile, asserting: Option<&'file Path>) -> Assertions<'file> {
        Assertions {
            file,
            kinds: asserting.map_or(&[][..], |_| Self::WATCHED),
            scopes: Scopes::default(),
            tables: MacroAliases::new(Self::TABLE),
            met: Vec::new(),
        }
    }

    fn judged(&self, invocation: &Macro, constants: &Bindings<'_>, locals: &Bindings<'_>) -> Vec<Violation<'file>> {
        let nested = |inner: &Macro| Macros::nested(self.file, inner).to_vec();
        let stops = |found: &Macro| self.assertion(found).compared(found).map(|_| ());
        let asserted = MacroTree::frontier(invocation, &nested, &stops);
        asserted.iter().filter_map(|found| self.violation(found, constants, locals)).collect()
    }

    fn assertion(&self, invocation: &Macro) -> Assertion {
        Assertion::named(&invocation.path.segments.last().map_or_default(|last| Name::from(&last.ident)), &self.tables)
    }

    fn violation(
        &self,
        invocation: &Macro,
        constants: &Bindings<'_>,
        locals: &Bindings<'_>,
    ) -> Option<Violation<'file>> {
        self.assertion(invocation).compared(invocation).and_then(|sides| {
            let line = invocation
                .path
                .segments
                .first()
                .map_or(LineNumber::FIRST, |first| LineNumber::from(first.ident.span()));
            let judge = Compared::new(constants, locals);
            let pinned = sides.iter().find_map(|side| judge.pinning(side));
            pinned.map(|pinning| Violation::new(Site::new(self.file.head().path(), line), Finding::from(pinning)))
        })
    }
}

impl<'ast> Watch<'ast> for Assertions<'_> {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_fn(&mut self, _: &'ast ItemFn, _: &Facts<'_, ItemFn>) -> Descend {
        self.scopes.opened()
    }

    fn off_item_fn(&mut self, _: &'ast ItemFn, _: &Facts<'_, ItemFn>) {
        self.scopes.shut();
    }

    fn on_impl_item_fn(&mut self, _: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Descend {
        self.scopes.opened()
    }

    fn off_impl_item_fn(&mut self, _: &'ast ImplItemFn, _: &Facts<'_, ImplItemFn>) {
        self.scopes.shut();
    }

    fn on_item_const(&mut self, item: &'ast ItemConst, _: &Facts<'_, ItemConst>) -> Descend {
        self.scopes.constant(Bound::constant(item));
        Descend::Into
    }

    fn on_local(&mut self, local: &'ast Local, _: &Facts<'_, Local>) -> Descend {
        self.scopes.local(Bound::local(local));
        Descend::Into
    }

    fn on_item_use(&mut self, item: &'ast ItemUse, facts: &Facts<'_, ItemUse>) -> Descend {
        self.tables.on_item_use(item, facts)
    }

    fn on_macro(&mut self, invocation: &'ast Macro, _: &Facts<'_, Macro>) -> Descend {
        self.met.push(Met { invocation: invocation.clone(), body: self.scopes.current() });
        Descend::Over
    }
}

impl<'file> Finds<'file> for Assertions<'file> {
    fn found(self: Box<Self>) -> Vec<Violation<'file>> {
        let constants = self.scopes.constants();
        self.met
            .iter()
            .flat_map(|met| self.judged(&met.invocation, &constants, &self.scopes.locals(met.body)))
            .collect()
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
            .or_else(|| (*name == Name::from(Self::ASSERT)).then_some(Assertion::Bare))
            .or_else(|| tables.call(name).map(|_| Assertion::Table))
            .unwrap_or(Assertion::Other)
    }

    fn compared(self, invocation: &Macro) -> Option<Vec<Expr>> {
        match self {
            Assertion::Equality => Some(Self::arguments(invocation).into_iter().take(Self::SIDES_LENGTH).collect()),
            Assertion::Bare => Some(Self::arguments(invocation).into_iter().take(1).flat_map(Self::sides).collect()),
            Assertion::Table => Some(TableRows::expected(&invocation.tokens)),
            Assertion::Other => None,
        }
    }

    fn arguments(invocation: &Macro) -> Vec<Expr> {
        invocation
            .parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
            .map_or_default(|arguments| arguments.into_iter().collect())
    }

    fn sides(condition: Expr) -> Vec<Expr> {
        match condition {
            Expr::Binary(binary) => Some(binary)
                .filter(|candidate| matches!(candidate.op, BinOp::Eq(_) | BinOp::Ne(_)))
                .map_or_default(|comparison| vec![*comparison.left, *comparison.right]),
            _ => Vec::new(),
        }
    }
}
