use lint_core::SourceFile;
mod members;
mod modules;
mod type_key;

use std::mem;
use std::path::Path;

use syntax::Punctuated;
use syntax::{Attribute, Ident, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemType, ItemUnion, Meta, MetaList, Token};

use lint_core::Module;
use lint_core::{Parsed, Site, Walk};
use lint_layout::{LineNumber, Name, Word};
use lint_sweep::{Descend, Facts, Gathers, Kind, Sweep, Watch, Watching};

use members::Members;
pub use modules::Modules;

#[derive(Debug)]
pub struct Guards<'file> {
    path: &'file Path,
    module: Module<'file>,
    members: Members<'file>,
    modules: Modules<'file>,
    outer: Vec<Frame<'file>>,
    kinds: &'static [Kind],
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
        Guards {
            path: head.path(),
            module: Module::of_head(head),
            members: Members::default(),
            modules: Modules::default(),
            outer: Vec::new(),
            kinds,
        }
    }

    fn declared(&mut self, attributes: &[Attribute], name: &Ident) -> Descend {
        let site = Site::new(self.path, LineNumber::from(name.span()));
        Descend::from(Walk::outside_tests(attributes))
            .entered()
            .map(|_| attributes.iter().find_map(MustUse::mark).map_or(Some(site), |_| None))
            .into_iter()
            .for_each(|mark| self.members.declare(Name::from(name), mark));
        Descend::Over
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

impl<'ast> Watch<'ast> for Guards<'_> {
    fn kinds(&self) -> &'static [Kind] {
        self.kinds
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        let inner = facts.name().map_or_else(|| self.module.clone(), |name| self.module.within(name.clone()));
        let outer = mem::replace(&mut self.module, inner);
        let around = mem::take(&mut self.members);
        self.outer.push(Frame { module: outer, members: around });
        Descend::from(facts.gate())
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod, _: &Facts<'_, ItemMod>) {
        let frame = self.outer.pop().unwrap_or(Frame { module: Module::UNPLACED, members: Members::default() });
        let visited = mem::replace(&mut self.module, frame.module);
        let within = mem::replace(&mut self.members, frame.members);
        self.modules.file(visited, within);
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        Descend::from(facts.gate())
            .entered()
            .into_iter()
            .for_each(|_| self.members.drop_implemented(Name::implementing(item, Word::new(Self::DROP))));
        Descend::Over
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct, _: &Facts<'_, ItemStruct>) -> Descend {
        self.declared(&item.attrs, &item.ident)
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum, _: &Facts<'_, ItemEnum>) -> Descend {
        self.declared(&item.attrs, &item.ident)
    }

    fn on_item_union(&mut self, item: &'ast ItemUnion, _: &Facts<'_, ItemUnion>) -> Descend {
        self.declared(&item.attrs, &item.ident)
    }

    fn on_item_type(&mut self, item: &'ast ItemType, facts: &Facts<'_, ItemType>) -> Descend {
        let target = Name::of_type(&item.ty);
        Descend::from(facts.gate())
            .entered()
            .map(|_| target)
            .into_iter()
            .zip(facts.name())
            .for_each(|(aliased, name)| self.members.alias(name.clone(), aliased));
        Descend::Over
    }
}

impl<'file> Gathers<Modules<'file>> for Guards<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        Sweep::alone(self)
    }

    fn gathered(self) -> Modules<'file> {
        let mut modules = self.modules;
        self.kinds
            .first()
            .map(|_| (self.module, self.members))
            .into_iter()
            .for_each(|(module, members)| modules.file(module, members));
        modules
    }
}
