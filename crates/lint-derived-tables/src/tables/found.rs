mod finding;

use std::collections::{BTreeMap, BTreeSet};

use algebra::{Monoid, Semigroup};
use lint_core::{Site, Violation};
use lint_file::Concluded;
use lint_layout::Name;

use finding::Finding;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Table {
    Listed(Name, BTreeSet<Name>),
    Spelled(Name),
    Mapped(Name),
}

#[derive(Debug, Default)]
pub struct Tables<'file> {
    enums: BTreeMap<Name, BTreeSet<BTreeSet<Name>>>,
    listings: Vec<Listing<'file>>,
    found: Vec<Violation<'file>>,
}

#[derive(Debug)]
struct Listing<'file> {
    site: Site<'file>,
    owner: Name,
    variants: BTreeSet<Name>,
}

impl<'file> Tables<'file> {
    pub fn declaring(owner: Name, variants: BTreeSet<Name>) -> Tables<'file> {
        Tables { enums: BTreeMap::from([(owner, BTreeSet::from([variants]))]), ..Tables::default() }
    }

    pub fn tabled(site: Site<'file>, table: Option<Table>) -> Tables<'file> {
        table.map_or_else(Tables::default, |found| Tables::of_table(site, found))
    }

    fn of_table(site: Site<'file>, table: Table) -> Tables<'file> {
        match table {
            Table::Listed(owner, variants) => {
                Tables { listings: vec![Listing { site, owner, variants }], ..Tables::default() }
            }
            Table::Spelled(owner) => Tables::flagged(site, Finding::Spelled(owner)),
            Table::Mapped(owner) => Tables::flagged(site, Finding::Mapped(owner)),
        }
    }

    fn flagged(site: Site<'file>, finding: Finding) -> Tables<'file> {
        Tables { found: vec![Violation::new(site, finding)], ..Tables::default() }
    }

    fn violations(self) -> Vec<Violation<'file>> {
        let enums = self.enums;
        let listed = self
            .listings
            .into_iter()
            .filter(|listing| enums.get(&listing.owner).is_some_and(|declared| declared.contains(&listing.variants)))
            .map(|listing| Violation::new(listing.site, Finding::Listed(listing.owner)));
        listed.chain(self.found).collect()
    }
}

impl Semigroup for Tables<'_> {
    fn combine(self, other: Self) -> Self {
        let enums = other.enums.into_iter().fold(self.enums, |mut enums, (owner, declared)| {
            enums.entry(owner).or_default().extend(declared);
            enums
        });
        Tables { enums, listings: self.listings.combine(other.listings), found: self.found.combine(other.found) }
    }
}

impl Monoid for Tables<'_> {}

impl<'file> FromIterator<Tables<'file>> for Concluded<'file> {
    fn from_iter<Parts: IntoIterator<Item = Tables<'file>>>(parts: Parts) -> Concluded<'file> {
        Monoid::concat(parts).violations().into_iter().collect()
    }
}
