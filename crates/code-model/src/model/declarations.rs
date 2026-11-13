use std::collections::BTreeMap;

use crate::boundary::ItemId;
use crate::domain::Domain;
use crate::item::{Alias, Item};
use crate::item_path::ItemPath;
use crate::resolved::Resolved;
use crate::sited::Sited;
use crate::type_ref::{Named, Rebuild, TypeRef};

#[derive(Debug)]
struct StdAlias {
    path: ItemPath,
    alias: Alias,
}

#[derive(Debug, Default)]
pub(crate) struct Aliases {
    declared: BTreeMap<ItemId, Alias>,
    std: Vec<StdAlias>,
}

impl Aliases {
    pub(crate) fn declare(&mut self, item: ItemId, alias: Alias) {
        self.declared.insert(item, alias);
    }

    pub(crate) fn declare_std(&mut self, path: ItemPath, alias: Alias) {
        self.std.push(StdAlias { path, alias });
    }

    pub(crate) fn expanded(&self, ty: &TypeRef) -> TypeRef {
        Expanding { aliases: self, steps: Steps::LIMIT }.child(ty).unwrap_or_else(|_| ty.clone())
    }

    pub(crate) fn aliasing(&self, path: &Resolved) -> Option<&Alias> {
        let declared = path.crate_item().and_then(|item| self.declared.get(&item));
        declared.or_else(|| {
            let std = path.std()?;
            self.std.iter().find(|row| row.path == *std).map(|row| &row.alias)
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Declarations {
    items: Vec<Item>,
    aliases: Aliases,
    exports: BTreeMap<Domain, BTreeMap<ItemPath, Resolved>>,
}

impl Declarations {
    pub(crate) fn new(
        items: Vec<Item>,
        aliases: Aliases,
        exports: BTreeMap<Domain, BTreeMap<ItemPath, Resolved>>,
    ) -> Declarations {
        Declarations { items, aliases, exports }
    }

    pub(crate) fn exports(&self) -> &BTreeMap<Domain, BTreeMap<ItemPath, Resolved>> {
        &self.exports
    }

    pub(crate) fn items(&self) -> &[Item] {
        &self.items
    }

    pub(crate) fn aliases(&self) -> &Aliases {
        &self.aliases
    }
}

#[derive(Clone, Copy, Debug)]
struct Steps(u8);

#[derive(Clone, Copy, Debug)]
struct Exhausted;

impl Steps {
    const LIMIT: Steps = Steps(16);

    fn taken(self) -> Result<Steps, Exhausted> {
        self.0.checked_sub(1).map(Steps).ok_or(Exhausted)
    }
}

#[derive(Clone, Copy, Debug)]
struct Expanding<'model> {
    aliases: &'model Aliases,
    steps: Steps,
}

impl Expanding<'_> {
    fn expansion(self, named: &Named<Sited>) -> Option<TypeRef> {
        self.aliases.aliasing(named.as_ref()).map(|alias| alias.expansion(named.arguments()))
    }
}

impl Rebuild<Sited> for Expanding<'_> {
    type Into = Sited;
    type Fail = Exhausted;

    fn path(&mut self, path: &Sited) -> Sited {
        path.clone()
    }

    fn child(&mut self, ty: &TypeRef) -> Result<TypeRef, Exhausted> {
        match ty.named().and_then(|named| self.expansion(named)) {
            Some(expansion) => Expanding { aliases: self.aliases, steps: self.steps.taken()? }.child(&expansion),
            None => ty.rebuilt(self),
        }
    }
}
