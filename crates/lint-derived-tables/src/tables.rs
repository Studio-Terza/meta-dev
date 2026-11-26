mod found;
mod shape;

use std::path::Path;

use syntax::{Block, Expr, Ident, ImplItem, ItemConst, ItemEnum, ItemFn, ItemImpl};

use algebra::Monoid;
use lint_core::{Parsed, Site, SourceFile};
use lint_layout::{LineNumber, Name};
use lint_sweep::{Around, Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind, Sweep};

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
    const KINDS: &'static [Kind] = &[Kind::ItemImpl, Kind::ItemEnum, Kind::ItemConst, Kind::ItemFn];

    fn site(&self, ident: &Ident) -> Site<'file> {
        Site::new(self.path, LineNumber::from(ident.span()))
    }

    fn tabled(&self, ident: &Ident, expr: &Expr, implementing: Option<&Name>) -> Tables<'file> {
        Tables::tabled(self.site(ident), Shape::within(implementing).table(expr))
    }

    fn mapped(&self, ident: &Ident, block: &Block, implementing: Option<&Name>) -> Tables<'file> {
        Tables::tabled(self.site(ident), Shape::within(implementing).mapped(block))
    }

    fn implemented(&self, item: &ImplItem, implementing: Option<&Name>) -> Tables<'file> {
        match item {
            ImplItem::Const(constant) => self.tabled(&constant.ident, &constant.expr, implementing),
            ImplItem::Fn(function) => self.mapped(&function.sig.ident, &function.block, implementing),
            _ => Tables::default(),
        }
    }
}

impl<'file> Contribute for TableScan<'file> {
    type Open = ();
    type Part = Tables<'file>;

    fn kinds(&self) -> &'static [Kind] {
        Self::KINDS
    }

    fn on_item_impl(
        &self,
        item: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
        _around: &Around<'_, Self::Open>,
    ) -> Contributed<(), Tables<'file>> {
        let implementing = Name::of_type(&item.self_ty);
        let found = Monoid::concat(item.items.iter().map(|inner| self.implemented(inner, implementing.as_ref())));
        Contributed::new((), found, Descend::Over)
    }

    fn on_item_enum(
        &self,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
        _around: &Around<'_, Self::Open>,
    ) -> Contributed<(), Tables<'file>> {
        let variants = item.variants.iter().map(|variant| Name::from(&variant.ident)).collect();
        Contributed::new((), Tables::declaring(Name::from(&item.ident), variants), Descend::Over)
    }

    fn on_item_const(
        &self,
        item: &ItemConst,
        _: &Facts<'_, ItemConst>,
        _around: &Around<'_, Self::Open>,
    ) -> Contributed<(), Tables<'file>> {
        Contributed::new((), self.tabled(&item.ident, &item.expr, None), Descend::Over)
    }

    fn on_item_fn(
        &self,
        item: &ItemFn,
        _: &Facts<'_, ItemFn>,
        _around: &Around<'_, Self::Open>,
    ) -> Contributed<(), Tables<'file>> {
        Contributed::new((), self.mapped(&item.sig.ident, &item.block, None), Descend::Into)
    }
}

impl<'file> Gathers<Tables<'file>> for Scanning<'file> {
    fn gathered(self, file: &SourceFile) -> Tables<'file> {
        let tested = self.tested;
        self.scan.concluded(file, |_, (), found| Monoid::concat([found].into_iter().chain(tested)))
    }
}
