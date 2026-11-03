mod declaration;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use lint_core::Site;
use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;
use type_facts::{Dependency, Facts};

use crate::link::Link;
use declaration::Declaration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Definition<'facts> {
    krate: &'facts CrateName,
    name: &'facts Name,
}

impl<'facts> From<&Declaration<'facts, '_>> for Definition<'facts> {
    fn from(declaration: &Declaration<'facts, '_>) -> Definition<'facts> {
        Definition { krate: declaration.krate(), name: declaration.name() }
    }
}

impl fmt::Display for Definition<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.krate, self.name)
    }
}

#[derive(Debug)]
pub(crate) struct Declared<'facts, 'file> {
    declaring: BTreeMap<&'facts Name, BTreeSet<&'facts CrateName>>,
    sites: BTreeMap<Definition<'facts>, Site<'file>>,
}

impl<'facts, 'file> Declared<'facts, 'file> {
    const EXTERNAL: [Word<'static>; 3] = words!["std", "core", "alloc"];

    pub(crate) fn of_facts(facts: &'facts Facts<'file>) -> Declared<'facts, 'file> {
        let sites: BTreeMap<Definition<'facts>, Site<'file>> = Declaration::all(facts)
            .map(|declaration| (Definition::from(&declaration), *AsRef::<Site<'file>>::as_ref(&declaration)))
            .collect();
        let declaring = sites.keys().fold(BTreeMap::new(), |mut declaring: BTreeMap<_, BTreeSet<_>>, node| {
            declaring.entry(node.name).or_default().insert(node.krate);
            declaring
        });
        Declared { declaring, sites }
    }

    pub(crate) fn site_of(&self, definition: &Definition<'facts>) -> Option<Site<'file>> {
        self.sites.get(definition).copied()
    }

    pub(crate) fn links(&self, facts: &'facts Facts<'file>) -> Vec<Link<'facts, 'file>> {
        Declaration::all(facts).flat_map(|declaration| self.links_of(declaration)).collect()
    }

    fn links_of(&self, declaration: Declaration<'facts, 'file>) -> impl Iterator<Item = Link<'facts, 'file>> {
        let from = Definition::from(&declaration);
        declaration.dependencies().iter().filter_map(move |dependency| {
            self.resolved(from.krate, dependency).map(|to| Link::new(from, to, dependency))
        })
    }

    fn resolved(&self, from: &CrateName, dependency: &Dependency<'_>) -> Option<Definition<'facts>> {
        let (name, declaring) = self.declaring.get_key_value(AsRef::<Name>::as_ref(dependency))?;
        let external = dependency.qualifier().and_then(|qualifier| qualifier.matches::<Whole>(&Self::EXTERNAL));
        let named = dependency.qualifier().and_then(|qualifier| Self::named(declaring, qualifier));
        let krate = external.map_or_else(|| named.or_else(|| Self::nearest(declaring, from)), |_| None);
        krate.map(|krate| Definition { krate, name })
    }

    fn named(declaring: &BTreeSet<&'facts CrateName>, qualifier: &Name) -> Option<&'facts CrateName> {
        declaring.iter().copied().find(|krate| Borrow::<Name>::borrow(*krate) == qualifier)
    }

    fn nearest(declaring: &BTreeSet<&'facts CrateName>, from: &CrateName) -> Option<&'facts CrateName> {
        declaring.get(from).or_else(|| declaring.first().filter(|_| declaring.len() == 1)).copied()
    }
}
