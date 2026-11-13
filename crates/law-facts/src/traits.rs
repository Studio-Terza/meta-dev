use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{ItemTrait, ItemUse, Visibility};

use crate::operators::Operators;
use crate::uses::{Place, Uses};
use lint_core::Parsed;
use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;

#[derive(Clone, Copy, Debug)]
struct Hops(usize);

impl Hops {
    fn next(self) -> Option<Hops> {
        self.0.checked_sub(1).map(Hops)
    }
}

#[derive(Debug, Default)]
struct Exports<'file> {
    declared: BTreeMap<&'file CrateName, BTreeSet<Name>>,
    exports: BTreeMap<&'file CrateName, Uses>,
}

impl Exports<'_> {
    fn exported(&self, owner: &CrateName, name: &Name, hops: Hops) -> Option<&Name> {
        self.declared
            .get(owner)
            .and_then(|declared| declared.get(name))
            .or_else(|| hops.next().and_then(|remaining| self.through(self.exports.get(owner), owner, name, remaining)))
    }

    fn through(&self, uses: Option<&Uses>, owner: &CrateName, name: &Name, hops: Hops) -> Option<&Name> {
        uses.into_iter().flat_map(|listed| listed.sources(name)).find_map(|place| {
            place.routed(|used| self.exported(owner, used, hops), |source, used| self.exported(source, used, hops))
        })
    }

    fn hops(&self) -> Hops {
        Hops(self.exports.len())
    }
}

#[derive(Debug, Default)]
pub struct Traits<'file> {
    exports: Exports<'file>,
    imports: BTreeMap<&'file CrateName, Uses>,
}

impl<'file> Traits<'file> {
    const CONVERSIONS: [Word<'static>; 4] = words!["From", "Into", "TryFrom", "TryInto"];

    pub(crate) fn declaring<'name>(
        &'name self,
        places: &[Place<'_>],
        named: &'name Name,
        home: &CrateName,
    ) -> Option<&'name Name> {
        places
            .iter()
            .find_map(|place| self.holding(place, home))
            .or_else(|| Operators::named(named))
            .or_else(|| Some(named).filter(|candidate| candidate.matches::<Whole>(&Self::CONVERSIONS).is_some()))
    }

    fn holding(&self, place: &Place<'_>, home: &CrateName) -> Option<&Name> {
        let hops = self.exports.hops();
        place.routed(|name| self.visible(home, name), |owner, name| self.exports.exported(owner, name, hops))
    }

    fn visible(&self, home: &CrateName, name: &Name) -> Option<&Name> {
        let hops = self.exports.hops();
        self.exports
            .exported(home, name, hops)
            .or_else(|| self.exports.through(self.imports.get(home), home, name, hops))
    }

    fn read(mut self, parsed: Parsed<'_, 'file>) -> Traits<'file> {
        let home = parsed.head().placement().package().crate_name();
        let mut declared = Declared::default();
        declared.visit_file(parsed.file().syntax());
        self.exports.declared.entry(home).or_default().extend(declared.traits);
        self.exports.exports.entry(home).or_default().absorb(declared.exports);
        self.imports.entry(home).or_default().absorb(declared.imports);
        parsed.test_modules().into_iter().fold(self, Traits::read)
    }

    fn absorbed(mut self, other: Traits<'file>) -> Traits<'file> {
        other
            .exports
            .declared
            .into_iter()
            .for_each(|(home, traits)| self.exports.declared.entry(home).or_default().extend(traits));
        other
            .exports
            .exports
            .into_iter()
            .for_each(|(home, uses)| self.exports.exports.entry(home).or_default().absorb(uses));
        other.imports.into_iter().for_each(|(home, uses)| self.imports.entry(home).or_default().absorb(uses));
        self
    }
}

#[derive(Debug, Default)]
struct Declared {
    traits: BTreeSet<Name>,
    exports: Uses,
    imports: Uses,
}

impl Declared {
    fn kept(&mut self, visibility: &Visibility) -> &mut Uses {
        match visibility {
            Visibility::Public(_) => &mut self.exports,
            Visibility::Restricted(_) | Visibility::Inherited => &mut self.imports,
        }
    }
}

impl<'ast> Visit<'ast> for Declared {
    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.traits.insert(Name::from(&item.ident));
        visit::visit_item_trait(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.kept(&item.vis).absorb(Uses::of_tree(&item.tree));
        visit::visit_item_use(self, item);
    }
}

impl<'head> From<Parsed<'_, 'head>> for Traits<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> Traits<'head> {
        Traits::default().read(parsed)
    }
}

impl<'head> FromIterator<Traits<'head>> for Traits<'head> {
    fn from_iter<Parts: IntoIterator<Item = Traits<'head>>>(parts: Parts) -> Traits<'head> {
        parts.into_iter().fold(Traits::default(), Traits::absorbed)
    }
}
