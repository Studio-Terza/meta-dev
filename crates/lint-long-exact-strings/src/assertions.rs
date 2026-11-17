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
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use bindings::{Bindings, BodyId, Bound, Scopes, Step};
use compared::Compared;
use finding::Finding;
use table_rows::TableRows;

#[derive(Debug)]
pub struct Assertions<'file> {
    file: &'file SourceFile,
    kinds: &'static [Kind],
}

#[derive(Debug, Default)]
pub struct Asserted<'file> {
    file: Option<&'file SourceFile>,
    scopes: Scopes,
    tables: Option<MacroAliases>,
    met: Vec<Met>,
}

#[derive(Debug)]
struct Met {
    invocation: Macro,
    body: Option<BodyId>,
}

#[derive(Debug)]
struct Judge<'run, 'file> {
    file: &'file SourceFile,
    tables: &'run MacroAliases,
}

impl<'file> Assertions<'file> {
    const TABLE: Word<'static> = Word::new("table");
    const WATCHED: &'static [Kind] =
        &[Kind::ItemFn, Kind::ImplItemFn, Kind::ItemConst, Kind::Local, Kind::ItemUse, Kind::Macro];

    pub fn new(file: &'file SourceFile, asserting: Option<&'file Path>) -> Assertions<'file> {
        Assertions { file, kinds: asserting.map_or(&[][..], |_| Self::WATCHED) }
    }
}

impl Asserted<'_> {
    fn tables(&self) -> MacroAliases {
        self.tables.clone().unwrap_or_else(|| MacroAliases::new(Assertions::TABLE))
    }
}

impl<'file> Judge<'_, 'file> {
    fn all(&self, asserted: &Asserted<'file>) -> Vec<Violation<'file>> {
        let constants = asserted.scopes.constants();
        let locals = |met: &Met| met.body.map_or_default(|id| id.locals(&asserted.scopes));
        asserted.met.iter().flat_map(|met| self.judged(&met.invocation, &constants, &locals(met))).collect()
    }

    fn judged(&self, invocation: &Macro, constants: &Bindings<'_>, locals: &Bindings<'_>) -> Vec<Violation<'file>> {
        let nested = |inner: &Macro| Macros::nested(self.file, inner).to_vec();
        let stops = |found: &Macro| self.assertion(found).compared(found).map(|_| ());
        let asserted = MacroTree::frontier(invocation, &nested, &stops);
        asserted.iter().filter_map(|found| self.violation(found, constants, locals)).collect()
    }

    fn assertion(&self, invocation: &Macro) -> Assertion {
        Assertion::named(&invocation.path.segments.last().map_or_default(|last| Name::from(&last.ident)), self.tables)
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
            pinned.map(|pinning| {
                Violation::new(Site::new(AsRef::<Path>::as_ref(self.file.head()), line), Finding::from(pinning))
            })
        })
    }
}

impl<'file> Fold for Assertions<'file> {
    type Found = Asserted<'file>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn finished(&self, found: Asserted<'file>) -> Asserted<'file> {
        Asserted { file: Some(self.file), ..found }
    }

    fn on_item_fn(&self, found: Asserted<'file>, _: &ItemFn, _: &Facts<'_, ItemFn>) -> Visit<Asserted<'file>> {
        let scopes = found.scopes.stepped(Step::Open);
        Visit::descending(Asserted { scopes, ..found })
    }

    fn off_item_fn(&self, found: Asserted<'file>, _: &ItemFn, _: &Facts<'_, ItemFn>) -> Asserted<'file> {
        let scopes = found.scopes.stepped(Step::Shut);
        Asserted { scopes, ..found }
    }

    fn on_impl_item_fn(
        &self,
        found: Asserted<'file>,
        _: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Asserted<'file>> {
        let scopes = found.scopes.stepped(Step::Open);
        Visit::descending(Asserted { scopes, ..found })
    }

    fn off_impl_item_fn(&self, found: Asserted<'file>, _: &ImplItemFn, _: &Facts<'_, ImplItemFn>) -> Asserted<'file> {
        let scopes = found.scopes.stepped(Step::Shut);
        Asserted { scopes, ..found }
    }

    fn on_item_const(
        &self,
        found: Asserted<'file>,
        item: &ItemConst,
        _: &Facts<'_, ItemConst>,
    ) -> Visit<Asserted<'file>> {
        let scopes = found.scopes.stepped(Step::Constant(Bound::constant(item)));
        Visit::descending(Asserted { scopes, ..found })
    }

    fn on_local(&self, found: Asserted<'file>, local: &Local, _: &Facts<'_, Local>) -> Visit<Asserted<'file>> {
        let scopes = found.scopes.stepped(Step::Local(Bound::local(local)));
        Visit::descending(Asserted { scopes, ..found })
    }

    fn on_item_use(&self, found: Asserted<'file>, item: &ItemUse, _: &Facts<'_, ItemUse>) -> Visit<Asserted<'file>> {
        let tables = found.tables().used(item);
        Visit::new(Asserted { tables: Some(tables), ..found }, Descend::Over)
    }

    fn on_macro(&self, found: Asserted<'file>, invocation: &Macro, _: &Facts<'_, Macro>) -> Visit<Asserted<'file>> {
        let met = Met { invocation: invocation.clone(), body: found.scopes.current() };
        Visit::new(Asserted { met: found.met.into_iter().chain([met]).collect(), ..found }, Descend::Over)
    }
}

impl<'file> From<Asserted<'file>> for Vec<Violation<'file>> {
    fn from(asserted: Asserted<'file>) -> Vec<Violation<'file>> {
        let tables = asserted.tables();
        asserted.file.map_or_default(|file| Judge { file, tables: &tables }.all(&asserted))
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
