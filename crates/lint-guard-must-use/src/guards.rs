use lint_core::SourceFile;
mod ignored;
mod members;
mod modules;
mod type_key;

use std::path::Path;

use syntax::Punctuated;
use syntax::{Attribute, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, ItemUnion, Meta, MetaList, Token};

use algebra::Monoid;
use lint_core::Module;
use lint_core::{Parsed, Site, Walk};
use lint_layout::{LineNumber, Name, Word};
use lint_sweep::{Contribute, Contributed, Contributing, Contribution, Descend, Facts, Gathers, Kind, Scope};

use crate::guards::ignored::Ignored;
use invariants::Ignoring;
use members::Members;
pub use modules::Modules;

#[derive(Debug)]
pub struct Guards<'file> {
    scan: Contributing<GuardScan<'file>>,
    module: Module<'file>,
}

#[derive(Debug)]
pub struct GuardScan<'file> {
    path: &'file Path,
    kinds: &'static [Kind],
}

#[derive(Debug)]
pub struct Nested<'file> {
    path: Vec<Name>,
    members: Members<'file>,
}

impl<'file> Nested<'file> {
    fn under(self, name: Option<&Name>) -> Nested<'file> {
        Nested { path: name.cloned().into_iter().chain(self.path).collect(), members: self.members }
    }

    fn filed(self, root: &Module<'file>) -> Modules<'file> {
        let module = self.path.into_iter().fold(root.clone(), |within, name| within.within(name));
        Modules::filed(module, self.members)
    }
}

impl<'file> Guards<'file> {
    const KINDS: &'static [Kind] =
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemStruct, Kind::ItemEnum, Kind::ItemUnion, Kind::ItemType];

    pub fn scanning(parsed: Parsed<'_, 'file>) -> Guards<'file> {
        let head = parsed.head();
        let kinds = <&SourceFile>::from(parsed).head().shipped().map_or(&[][..], |_| Self::KINDS);
        let scan = Contributing::new(GuardScan { path: AsRef::<Path>::as_ref(head), kinds });
        Guards { scan, module: Module::of_head(head) }
    }
}

impl<'file> GuardScan<'file> {
    const DROP: &str = "Drop";

    fn declared(&self, attributes: &[Attribute], name: &Ident) -> Contributed<Members<'file>, Vec<Nested<'file>>> {
        let site = Site::new(self.path, LineNumber::from(name.span()));
        let members = Descend::from(Walk::outside_tests(attributes))
            .entered()
            .map(|()| attributes.iter().find_map(MustUse::mark).map_or(Some(site), |_| None))
            .map_or_else(Members::default, |unmarked| Members::declaring(Name::from(name), unmarked));
        Contributed::new(members, Vec::new(), Descend::Over)
    }
}

struct MustUse;

impl MustUse {
    const MUST_USE: &str = "must_use";
    const CONDITIONAL: &str = "cfg_attr";

    fn mark(attribute: &Attribute) -> Option<&Attribute> {
        Some(attribute).filter(|candidate| candidate.path().is_ident(Self::MUST_USE)).or_else(|| {
            attribute
                .meta
                .require_list()
                .ignoring(Ignored::Unparsed)
                .filter(|list| list.path.is_ident(Self::CONDITIONAL))
                .and_then(Self::conditional)
                .map(|_| attribute)
        })
    }

    fn conditional(list: &MetaList) -> Option<Meta> {
        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ignoring(Ignored::Unparsed)
            .and_then(|nested| nested.into_iter().skip(1).find(|meta| meta.path().is_ident(Self::MUST_USE)))
    }
}

impl<'file> Contribute for GuardScan<'file> {
    type Open = Members<'file>;
    type Part = Vec<Nested<'file>>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn scopes(&self) -> &'static [Scope] {
        const { &[Scope::new(Kind::ItemMod)] }
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Members<'file>, Vec<Nested<'file>>> {
        Contributed::descending(Descend::from(facts.gate()))
    }

    fn off_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        inner: Contribution<Members<'file>, Vec<Nested<'file>>>,
    ) -> Vec<Nested<'file>> {
        inner.concluded(|members, nested| {
            let own = Nested { path: Vec::new(), members };
            [own].into_iter().chain(nested).map(|within| within.under(facts.name())).collect()
        })
    }

    fn on_item_impl(
        &self,
        item: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Members<'file>, Vec<Nested<'file>>> {
        let dropped = Descend::from(facts.gate())
            .entered()
            .map_or_else(Members::default, |()| Members::dropping(Name::implementing(item, Word::new(Self::DROP))));
        Contributed::new(dropped, Vec::new(), Descend::Over)
    }

    fn on_item_struct(
        &self,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Members<'file>, Vec<Nested<'file>>> {
        self.declared(&item.attrs, &item.ident)
    }

    fn on_item_enum(
        &self,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Members<'file>, Vec<Nested<'file>>> {
        self.declared(&item.attrs, &item.ident)
    }

    fn on_item_union(
        &self,
        item: &ItemUnion,
        _: &Facts<'_, ItemUnion>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Members<'file>, Vec<Nested<'file>>> {
        self.declared(&item.attrs, &item.ident)
    }

    fn on_item_type(
        &self,
        item: &ItemType,
        facts: &Facts<'_, ItemType>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<Members<'file>, Vec<Nested<'file>>> {
        let target = Name::of_type(&item.ty);
        let aliased = Descend::from(facts.gate()).entered().and(facts.name());
        let members = aliased.map_or_else(Members::default, |name| Members::aliasing(name.clone(), target));
        Contributed::new(members, Vec::new(), Descend::Over)
    }
}

impl<'file> Gathers<Modules<'file>> for Guards<'file> {
    fn gathered(self, file: &SourceFile) -> Modules<'file> {
        let module = self.module;
        self.scan.concluded(file, |scan, members, nested| {
            let own = scan.kinds.first().map(|_| Nested { path: Vec::new(), members });
            Monoid::concat(own.into_iter().chain(nested).map(|within| within.filed(&module)))
        })
    }
}
