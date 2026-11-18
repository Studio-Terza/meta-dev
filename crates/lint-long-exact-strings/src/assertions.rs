mod bindings;
mod compared;
mod finding;
mod ignored;
mod pinning;
mod table_rows;

use std::path::Path;

use lint_core::{Macros, SourceFile};

use syntax::Punctuated;
use syntax::{BinOp, Expr, ImplItemFn, ItemConst, ItemFn, ItemUse, Local, Macro, MacroTree, Token};

use lint_core::{Aliased, MacroAliases, Site, Violation};
use lint_layout::{LineNumber, Name, Whole, Word, words};
use lint_sweep::{Concludes, Contribute, Contributed, Contribution, Descend, Facts, Kind, Scope};

use crate::assertions::ignored::Ignored;
use bindings::{Asserted, Bindings, Judging, Met};
use compared::Compared;
use finding::Finding;
use invariants::Ignoring;
use table_rows::TableRows;

#[derive(Debug)]
pub struct Assertions<'file> {
    file: &'file SourceFile,
    kinds: &'static [Kind],
    tables: MacroAliases,
}

impl<'file> Assertions<'file> {
    const TABLE: Word<'static> = Word::new("table");
    const WATCHED: &'static [Kind] =
        &[Kind::ItemFn, Kind::ImplItemFn, Kind::ItemConst, Kind::Local, Kind::ItemUse, Kind::Macro];

    pub fn new(file: &'file SourceFile, asserting: Option<&'file Path>) -> Assertions<'file> {
        Assertions { file, kinds: asserting.map_or(&[][..], |_| Self::WATCHED), tables: MacroAliases::new(Self::TABLE) }
    }
}

impl<'file> Judge<'file> {
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
            pinned.map(|pinning| {
                Violation::new(Site::new(AsRef::<Path>::as_ref(self.file.head()), line), Finding::from(pinning))
            })
        })
    }
}

impl Contribute for Assertions<'_> {
    type Open = ();
    type Part = Asserted;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemFn), Scope::new(Kind::ImplItemFn)] }
    }

    fn off_item_fn(&self, _: &ItemFn, _: &Facts<'_, ItemFn>, inner: Contribution<(), Asserted>) -> Asserted {
        inner.concluded(|(), asserted| asserted.bodied())
    }

    fn off_impl_item_fn(
        &self,
        _: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        inner: Contribution<(), Asserted>,
    ) -> Asserted {
        inner.concluded(|(), asserted| asserted.bodied())
    }

    fn on_item_const(&self, item: &ItemConst, _: &Facts<'_, ItemConst>) -> Contributed<(), Asserted> {
        Contributed::parted(Asserted::constant(item))
    }

    fn on_local(&self, local: &Local, _: &Facts<'_, Local>) -> Contributed<(), Asserted> {
        Contributed::parted(Asserted::local(local))
    }

    fn on_item_use(&self, item: &ItemUse, _: &Facts<'_, ItemUse>) -> Contributed<(), Asserted> {
        Contributed::new((), Asserted::aliased(self.tables.renamed(&item.tree)), Descend::Over)
    }

    fn on_macro(&self, invocation: &Macro, _: &Facts<'_, Macro>) -> Contributed<(), Asserted> {
        Contributed::new((), Asserted::met(invocation), Descend::Over)
    }
}

impl<'file> Concludes<'file> for Assertions<'file> {
    fn violations(self, (): (), asserted: Asserted) -> Vec<Violation<'file>> {
        asserted.concluded(|aliases, judging| {
            let Assertions { file, tables, .. } = self;
            Judge { file, tables: tables.aliased(aliases) }.all(&judging)
        })
    }
}

impl<'file> Judge<'file> {
    pub fn all(&self, judging: &Judging) -> Vec<Violation<'file>> {
        let constants = judging.constants();
        let mut found: Vec<Violation<'file>> = judging.met().flat_map(|met| self.met(met, &constants)).collect();
        found.sort_by_key(|violation| *violation.site());
        found
    }

    fn met(&self, met: Met<'_>, constants: &Bindings<'_>) -> Vec<Violation<'file>> {
        met.judged(|invocation, locals| self.judged(invocation, constants, &locals))
    }
}

#[derive(Debug)]
struct Judge<'file> {
    file: &'file SourceFile,
    tables: Aliased,
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

    fn named(name: &Name, tables: &Aliased) -> Assertion {
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
            .ignoring(Ignored::Unparsed)
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
