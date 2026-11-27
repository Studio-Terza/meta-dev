use std::collections::BTreeSet;
use std::fmt;
use std::ops::Add;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, Field, ImplItemConst, ImplItemFn, ImplItemType, Item, Macro, TraitItemFn, TraitItemType};

use crate::gate::Gate;
use crate::source::{Place, Source};
use crate::terms::{Count, Depth, Name};
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Budget {
    lines: Count,
    test_lines: Count,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TestScan {
    test_lines: Count,
    gated: Depth,
    tested: BTreeSet<Name>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Exceeded(Budget);

impl Budget {
    const PERCENT_OF_LINES: usize = 18;

    pub(crate) fn of_file(source: &Source, scan: &TestScan, gate: Gate) -> Budget {
        let lines = source.text().lines();
        let shipped = gate.entered().filter(|()| source.place() == Place::Shipped).and(source.package().shipped());
        Budget { lines, test_lines: shipped.map_or(lines, |()| scan.test_lines) }
    }

    pub(crate) fn exceeded(self) -> Option<Exceeded> {
        self.test_lines.over_share(self.lines, Budget::PERCENT_OF_LINES).map(|()| Exceeded(self))
    }
}

impl TestScan {
    pub(crate) fn within(tested: &BTreeSet<Name>) -> TestScan {
        TestScan { tested: tested.clone(), ..TestScan::default() }
    }

    fn skipped(&self, attributes: &[Attribute]) -> Option<()> {
        Gate::of(attributes).and(Gate::featured(attributes, &self.tested)).entered().xor(Some(()))
    }

    fn entered(&mut self, attributes: &[Attribute], span: Span) -> Descent {
        let skipped = self.skipped(attributes);
        let outermost = skipped.and(self.gated.surface()).map(|()| Count::spanning(span));
        self.test_lines = self.test_lines.plus(outermost.unwrap_or_default());
        self.gated = skipped.map_or(self.gated, |()| self.gated.deeper());
        Descent::Into
    }

    fn left(&mut self, attributes: &[Attribute]) {
        self.gated = self.skipped(attributes).map_or(self.gated, |()| self.gated.shallower());
    }

    fn attributes(item: &Item) -> &[Attribute] {
        match item {
            Item::Const(inner) => &inner.attrs,
            Item::Enum(inner) => &inner.attrs,
            Item::ExternCrate(inner) => &inner.attrs,
            Item::Fn(inner) => &inner.attrs,
            Item::ForeignMod(inner) => &inner.attrs,
            Item::Impl(inner) => &inner.attrs,
            Item::Macro(inner) => &inner.attrs,
            Item::Mod(inner) => &inner.attrs,
            Item::Static(inner) => &inner.attrs,
            Item::Struct(inner) => &inner.attrs,
            Item::Trait(inner) => &inner.attrs,
            Item::TraitAlias(inner) => &inner.attrs,
            Item::Type(inner) => &inner.attrs,
            Item::Union(inner) => &inner.attrs,
            Item::Use(inner) => &inner.attrs,
            _ => &[],
        }
    }
}

impl Hooks for TestScan {
    fn item(&mut self, item: &Item) -> Descent {
        self.entered(TestScan::attributes(item), item.span())
    }

    fn left_item(&mut self, item: &Item) {
        self.left(TestScan::attributes(item));
    }

    fn impl_item_fn(&mut self, item: &ImplItemFn) -> Descent {
        self.entered(&item.attrs, item.span())
    }

    fn left_impl_item_fn(&mut self, item: &ImplItemFn) {
        self.left(&item.attrs);
    }

    fn impl_item_const(&mut self, item: &ImplItemConst) -> Descent {
        self.entered(&item.attrs, item.span())
    }

    fn left_impl_item_const(&mut self, item: &ImplItemConst) {
        self.left(&item.attrs);
    }

    fn impl_item_type(&mut self, item: &ImplItemType) -> Descent {
        self.entered(&item.attrs, item.span())
    }

    fn left_impl_item_type(&mut self, item: &ImplItemType) {
        self.left(&item.attrs);
    }

    fn trait_item_fn(&mut self, item: &TraitItemFn) -> Descent {
        self.entered(&item.attrs, item.span())
    }

    fn left_trait_item_fn(&mut self, item: &TraitItemFn) {
        self.left(&item.attrs);
    }

    fn trait_item_type(&mut self, item: &TraitItemType) -> Descent {
        self.entered(&item.attrs, item.span())
    }

    fn left_trait_item_type(&mut self, item: &TraitItemType) {
        self.left(&item.attrs);
    }

    fn field(&mut self, field: &Field) -> Descent {
        self.entered(&field.attrs, field.span())
    }

    fn left_field(&mut self, field: &Field) {
        self.left(&field.attrs);
    }

    fn attribute(&mut self, attribute: &Attribute) -> Descent {
        let conditional = Gate::conditional(attribute, &self.tested).entered().xor(Some(()));
        let counted = conditional.and(self.gated.surface()).map(|()| Count::spanning(attribute.span()));
        self.test_lines = self.test_lines.plus(counted.unwrap_or_default());
        Descent::Into
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Nothing
    }
}

impl Add for Budget {
    type Output = Budget;

    fn add(self, other: Budget) -> Budget {
        Budget { lines: self.lines.plus(other.lines), test_lines: self.test_lines.plus(other.test_lines) }
    }
}

impl fmt::Display for Budget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "test budget: {} of {} lines are tests, at most {}% may be",
            self.test_lines,
            self.lines,
            Budget::PERCENT_OF_LINES
        )
    }
}

impl fmt::Display for Exceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}; cut test lines or test through fewer, table-driven cases", self.0)
    }
}
