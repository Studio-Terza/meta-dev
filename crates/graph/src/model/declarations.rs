use std::collections::BTreeMap;
use std::ops::Range;

use algebra::{Monoid, Semigroup};
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::{btree_map, vec};
use newtype::proptest::strategy::Strategy;

use crate::boundary::ItemId;
use crate::domain::Domain;
use crate::item::{Alias, Item};
use crate::item_path::ItemPath;
use crate::resolved::Resolved;
use crate::sited::Sited;
use crate::type_ref::{Named, Rebuild, TypeRef};

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
struct StdAlias {
    path: ItemPath,
    alias: Alias,
}

#[derive(newtype::Arbitrary, Clone, Debug, Default, PartialEq, Eq)]
#[newtype(generate = Aliases::sampled)]
pub struct Aliases {
    declared: BTreeMap<ItemId, Alias>,
    std: Vec<StdAlias>,
}

impl Semigroup for Aliases {
    fn combine(self, other: Aliases) -> Aliases {
        let mut declared = self.declared;
        declared.extend(other.declared);
        Aliases { declared, std: self.std.combine(other.std) }
    }
}

impl Monoid for Aliases {}

impl Aliases {
    const SAMPLED: Range<usize> = 0..4;

    fn sampled() -> impl Strategy<Value = Aliases> {
        Strategy::prop_map(
            (btree_map(any::<ItemId>(), any::<Alias>(), Aliases::SAMPLED), vec(any::<StdAlias>(), Aliases::SAMPLED)),
            |(declared, std)| Aliases { declared, std },
        )
    }

    #[must_use]
    pub fn declaring(item: ItemId, alias: Alias) -> Aliases {
        Aliases { declared: BTreeMap::from([(item, alias)]), std: Vec::new() }
    }

    #[must_use]
    pub fn std(path: ItemPath, alias: Alias) -> Aliases {
        Aliases { declared: BTreeMap::new(), std: vec![StdAlias { path, alias }] }
    }

    #[must_use]
    pub fn target(&self, path: &Resolved) -> Option<&Named<Sited>> {
        self.aliasing(path)?.target().named()
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

#[derive(newtype::Arbitrary, Clone, Debug, Default, PartialEq, Eq)]
#[newtype(generate = Exports::sampled)]
pub struct Exports {
    domains: BTreeMap<Domain, BTreeMap<ItemPath, Resolved>>,
}

impl Semigroup for Exports {
    fn combine(self, other: Exports) -> Exports {
        let mut domains = self.domains;
        other.domains.into_iter().for_each(|(domain, exported)| domains.entry(domain).or_default().extend(exported));
        Exports { domains }
    }
}

impl Monoid for Exports {}

impl Exports {
    const SAMPLED: Range<usize> = 0..4;

    fn sampled() -> impl Strategy<Value = Exports> {
        let exported = btree_map(any::<ItemPath>(), any::<Resolved>(), Self::SAMPLED);
        Strategy::prop_map(btree_map(any::<Domain>(), exported, Self::SAMPLED), |domains| Exports { domains })
    }

    #[must_use]
    pub fn export(domain: Domain, path: ItemPath, resolved: &Resolved) -> Exports {
        let exported = resolved.known().map(|known| (path, known.clone())).into_iter().collect();
        Exports { domains: BTreeMap::from([(domain, exported)]) }
    }
}

#[derive(Debug, Default)]
pub struct Issued {
    items: Vec<Item>,
    scopes: Vec<Vec<Option<ItemId>>>,
}

impl FromIterator<Vec<Option<Item>>> for Issued {
    fn from_iter<Scopes: IntoIterator<Item = Vec<Option<Item>>>>(scopes: Scopes) -> Issued {
        let declared: Vec<Vec<Option<Item>>> = scopes.into_iter().collect();
        let numbered = ItemId::numbered(&declared);
        Issued { items: declared.into_iter().flatten().flatten().collect(), scopes: numbered }
    }
}

impl Issued {
    #[must_use]
    pub fn scopes(&self) -> &[Vec<Option<ItemId>>] {
        &self.scopes
    }
}

#[derive(Debug, Default)]
pub struct Declarations {
    items: Vec<Item>,
    aliases: Aliases,
    exports: BTreeMap<Domain, BTreeMap<ItemPath, Resolved>>,
}

impl Declarations {
    #[must_use]
    pub fn issued(issued: Issued, aliases: Aliases, exports: Exports) -> Declarations {
        Declarations { items: issued.items, aliases, exports: exports.domains }
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

    fn path(&self, path: &Sited) -> Sited {
        path.clone()
    }

    fn child(&self, ty: &TypeRef) -> Result<TypeRef, Exhausted> {
        match ty.named().and_then(|named| self.expansion(named)) {
            Some(expansion) => Expanding { aliases: self.aliases, steps: self.steps.taken()? }.child(&expansion),
            None => ty.rebuilt(self),
        }
    }
}
