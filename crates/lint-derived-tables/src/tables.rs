mod found;
mod shape;

use std::path::Path;

use syntax::{Block, Expr, Ident, ImplItemConst, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemImpl};

use algebra::{Monoid, Semigroup};
use lint_core::{Parsed, Site, SourceFile};
use lint_layout::{LineNumber, Name};
use lint_sweep::{Around, Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind, Scope, Sweep};

pub use found::Tables;
use shape::Shape;

#[derive(Debug)]
pub struct Scanning<'file> {
    scan: Contributing<TableScan<'file>>,
    tested: Vec<Tables<'file>>,
}

#[derive(Debug)]
struct TableScan<'file> {
    path: &'file Path,
}

#[derive(Debug, Default)]
struct Implementing(Option<Name>);

impl Semigroup for Implementing {
    fn combine(self, other: Implementing) -> Implementing {
        Implementing(self.0.or(other.0))
    }
}

impl Monoid for Implementing {}

impl<'file> Scanning<'file> {
    pub fn reading(parsed: Parsed<'_, 'file>) -> Scanning<'file> {
        let tested = parsed
            .test_modules()
            .into_iter()
            .map(|module| Sweep::gathered(<&SourceFile>::from(module), Scanning::reading(module)))
            .collect();
        Scanning { scan: Contributing::new(TableScan { path: AsRef::<Path>::as_ref(parsed.head()) }), tested }
    }
}

impl<'file> TableScan<'file> {
    const KINDS: &'static [Kind] =
        &[Kind::ItemImpl, Kind::ItemEnum, Kind::ItemConst, Kind::ImplItemConst, Kind::ItemFn, Kind::ImplItemFn];

    fn site(&self, ident: &Ident) -> Site<'file> {
        Site::new(self.path, LineNumber::from(ident.span()))
    }

    fn tabled(
        &self,
        ident: &Ident,
        expr: &Expr,
        around: &Around<'_, Implementing>,
    ) -> Contributed<Implementing, Tables<'file>> {
        let found = Shape::within(Self::implementing(around)).table(expr);
        Contributed::new(Implementing::default(), Tables::tabled(self.site(ident), found), Descend::Over)
    }

    fn mapped(
        &self,
        ident: &Ident,
        block: &Block,
        around: &Around<'_, Implementing>,
    ) -> Contributed<Implementing, Tables<'file>> {
        let found = Shape::within(Self::implementing(around)).mapped(block);
        Contributed::new(Implementing::default(), Tables::tabled(self.site(ident), found), Descend::Into)
    }

    fn implementing<'around>(around: &'around Around<'_, Implementing>) -> Option<&'around Name> {
        around.nearest().find_map(|open| open.0.as_ref())
    }
}

impl<'file> Contribute for TableScan<'file> {
    type Open = Implementing;
    type Part = Tables<'file>;

    fn kinds(&self) -> &'static [Kind] {
        Self::KINDS
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemImpl)] }
    }

    fn on_item_impl(
        &self,
        item: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        _around: &Around<'_, Self::Open>,
    ) -> Contributed<Implementing, Tables<'file>> {
        Contributed::new(Implementing(Name::of_type(&item.self_ty)), Tables::default(), Descend::Into)
    }

    fn on_item_enum(
        &self,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
        _around: &Around<'_, Self::Open>,
    ) -> Contributed<Implementing, Tables<'file>> {
        let variants = item.variants.iter().map(|variant| Name::from(&variant.ident)).collect();
        Contributed::new(Implementing::default(), Tables::declaring(Name::from(&item.ident), variants), Descend::Over)
    }

    fn on_item_const(
        &self,
        item: &ItemConst,
        _: &Facts<'_, ItemConst>,
        around: &Around<'_, Self::Open>,
    ) -> Contributed<Implementing, Tables<'file>> {
        self.tabled(&item.ident, &item.expr, around)
    }

    fn on_impl_item_const(
        &self,
        item: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
        around: &Around<'_, Self::Open>,
    ) -> Contributed<Implementing, Tables<'file>> {
        self.tabled(&item.ident, &item.expr, around)
    }

    fn on_item_fn(
        &self,
        item: &ItemFn,
        _: &Facts<'_, ItemFn>,
        around: &Around<'_, Self::Open>,
    ) -> Contributed<Implementing, Tables<'file>> {
        self.mapped(&item.sig.ident, &item.block, around)
    }

    fn on_impl_item_fn(
        &self,
        item: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
        around: &Around<'_, Self::Open>,
    ) -> Contributed<Implementing, Tables<'file>> {
        self.mapped(&item.sig.ident, &item.block, around)
    }
}

impl<'file> Gathers<Tables<'file>> for Scanning<'file> {
    fn gathered(self, file: &SourceFile) -> Tables<'file> {
        let tested = self.tested;
        self.scan.concluded(file, |_, _, found| Monoid::concat([found].into_iter().chain(tested)))
    }
}
