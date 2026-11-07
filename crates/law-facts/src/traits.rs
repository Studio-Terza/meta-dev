use std::collections::{BTreeMap, BTreeSet};

use syn::visit::{self, Visit};
use syn::{ItemTrait, ItemUse, Visibility};

use crate::operators::Operators;
use crate::uses::{Place, Uses};
use lint_core::SourceFile;
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
            .or_else(|| hops.next().and_then(|hops| self.through(self.exports.get(owner), owner, name, hops)))
    }

    fn through(&self, uses: Option<&Uses>, owner: &CrateName, name: &Name, hops: Hops) -> Option<&Name> {
        uses.into_iter().flat_map(|uses| uses.sources(name)).find_map(|place| {
            place.routed(|used| self.exported(owner, used, hops), |source, used| self.exported(source, used, hops))
        })
    }

    fn hops(&self) -> Hops {
        Hops(self.exports.len())
    }
}

#[derive(Debug, Default)]
pub(crate) struct Traits<'file> {
    exports: Exports<'file>,
    imports: BTreeMap<&'file CrateName, Uses>,
}

impl<'file> Traits<'file> {
    const CONVERSIONS: [Word<'static>; 4] = words!["From", "Into", "TryFrom", "TryInto"];

    pub(crate) fn declared(files: &'file [SourceFile]) -> Traits<'file> {
        files.iter().fold(Traits::default(), Traits::read)
    }

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
            .or_else(|| Some(named).filter(|named| named.matches::<Whole>(&Self::CONVERSIONS).is_some()))
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

    fn read(mut self, file: &'file SourceFile) -> Traits<'file> {
        let home = file.placement().package().crate_name();
        let mut declared = Declared::default();
        declared.visit_file(file.syntax());
        self.exports.declared.entry(home).or_default().extend(declared.traits);
        self.exports.exports.entry(home).or_default().absorb(declared.exports);
        self.imports.entry(home).or_default().absorb(declared.imports);
        file.test_modules().iter().fold(self, Traits::read)
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
