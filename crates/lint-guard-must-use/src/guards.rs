use lint_core::SourceFile;
mod members;
mod modules;
mod type_key;

use std::path::Path;

use syntax::Punctuated;
use syntax::{Attribute, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, ItemUnion, Meta, MetaList, Token};

use lint_core::Module;
use lint_core::{Parsed, Site, Walk};
use lint_file::Concluded;
use lint_layout::{LineNumber, Name, Word};
use lint_sweep::{Descend, Facts, Fold, Kind, Visit};

use members::{Dropped, Members};
use modules::Modules;

#[derive(Debug)]
pub struct Guards<'file> {
    path: &'file Path,
    module: Module<'file>,
    kinds: &'static [Kind],
}

#[derive(Debug, Default)]
pub struct Scanned<'file> {
    module: Option<Module<'file>>,
    members: Members<'file>,
    modules: Modules<'file>,
    outer: Vec<Frame<'file>>,
}

#[derive(Debug)]
struct Frame<'file> {
    module: Module<'file>,
    members: Members<'file>,
}

impl<'file> Guards<'file> {
    const DROP: &str = "Drop";
    const KINDS: &'static [Kind] =
        &[Kind::ItemMod, Kind::ItemImpl, Kind::ItemStruct, Kind::ItemEnum, Kind::ItemUnion, Kind::ItemType];

    pub fn scanning(parsed: Parsed<'_, 'file>) -> Guards<'file> {
        let head = parsed.head();
        let kinds = <&SourceFile>::from(parsed).shipped().map_or(&[][..], |_| Self::KINDS);
        Guards { path: AsRef::<Path>::as_ref(head), module: Module::of_head(head), kinds }
    }

    fn declared(&self, found: Scanned<'file>, attributes: &[Attribute], name: &Ident) -> Visit<Scanned<'file>> {
        let site = Site::new(self.path, LineNumber::from(name.span()));
        let marked = Descend::from(Walk::outside_tests(attributes))
            .entered()
            .map(|_| attributes.iter().find_map(MustUse::mark).map_or(Some(site), |_| None));
        let members = marked.into_iter().fold(found.members, |members, mark| members.declared(Name::from(name), mark));
        Visit::new(Scanned { members, ..found }, Descend::Over)
    }

    fn current(&self, found: &Scanned<'file>) -> Module<'file> {
        found.module.clone().unwrap_or_else(|| self.module.clone())
    }

    fn topmost(&self, modules: Modules<'file>, top: Module<'file>, members: Members<'file>) -> Modules<'file> {
        match self.kinds.first() {
            Some(_) => modules.filed(top, members),
            None => modules,
        }
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
                .ok()
                .filter(|list| list.path.is_ident(Self::CONDITIONAL))
                .and_then(Self::conditional)
                .map(|_| attribute)
        })
    }

    fn conditional(list: &MetaList) -> Option<Meta> {
        list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ok()
            .and_then(|nested| nested.into_iter().skip(1).find(|meta| meta.path().is_ident(Self::MUST_USE)))
    }
}

impl<'file> Fold for Guards<'file> {
    type Found = Scanned<'file>;

    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn finished(&self, found: Scanned<'file>) -> Scanned<'file> {
        let top = self.current(&found);
        let Scanned { members, modules, outer, .. } = found;
        Scanned { module: None, members: Members::default(), modules: self.topmost(modules, top, members), outer }
    }

    fn on_item_mod(&self, found: Scanned<'file>, _: &ItemMod, facts: &Facts<'_, ItemMod>) -> Visit<Scanned<'file>> {
        let current = self.current(&found);
        let inner = facts.name().map_or_else(|| current.clone(), |name| current.within(name.clone()));
        let Scanned { members, modules, outer, .. } = found;
        let frame = Frame { module: current, members };
        let opened = outer.into_iter().chain([frame]).collect();
        Visit::new(
            Scanned { module: Some(inner), members: Members::default(), modules, outer: opened },
            Descend::from(facts.gate()),
        )
    }

    fn off_item_mod(&self, found: Scanned<'file>, _: &ItemMod, _: &Facts<'_, ItemMod>) -> Scanned<'file> {
        let visited = self.current(&found);
        let Scanned { members, modules, mut outer, .. } = found;
        let frame = outer.pop().unwrap_or(Frame { module: Module::UNPLACED, members: Members::default() });
        Scanned { module: Some(frame.module), members: frame.members, modules: modules.filed(visited, members), outer }
    }

    fn on_item_impl(
        &self,
        found: Scanned<'file>,
        item: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
    ) -> Visit<Scanned<'file>> {
        let dropped =
            Descend::from(facts.gate()).entered().and_then(|_| Name::implementing(item, Word::new(Self::DROP)));
        Visit::new(Scanned { members: found.members.drop_implemented(Dropped::from(dropped)), ..found }, Descend::Over)
    }

    fn on_item_struct(
        &self,
        found: Scanned<'file>,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
    ) -> Visit<Scanned<'file>> {
        self.declared(found, &item.attrs, &item.ident)
    }

    fn on_item_enum(&self, found: Scanned<'file>, item: &ItemEnum, _: &Facts<'_, ItemEnum>) -> Visit<Scanned<'file>> {
        self.declared(found, &item.attrs, &item.ident)
    }

    fn on_item_union(
        &self,
        found: Scanned<'file>,
        item: &ItemUnion,
        _: &Facts<'_, ItemUnion>,
    ) -> Visit<Scanned<'file>> {
        self.declared(found, &item.attrs, &item.ident)
    }

    fn on_item_type(
        &self,
        found: Scanned<'file>,
        item: &ItemType,
        facts: &Facts<'_, ItemType>,
    ) -> Visit<Scanned<'file>> {
        let target = Descend::from(facts.gate()).entered().and_then(|_| Name::of_type(&item.ty));
        let members =
            facts.name().into_iter().fold(found.members, |members, name| members.aliased(name.clone(), target.clone()));
        Visit::new(Scanned { members, ..found }, Descend::Over)
    }
}

impl<'file> FromIterator<Scanned<'file>> for Concluded<'file> {
    fn from_iter<Parts: IntoIterator<Item = Scanned<'file>>>(parts: Parts) -> Concluded<'file> {
        parts.into_iter().map(|scanned| scanned.modules).collect::<Modules<'file>>().violations().into_iter().collect()
    }
}
