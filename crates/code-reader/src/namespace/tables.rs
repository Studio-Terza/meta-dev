mod components;

use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

use lint_layout::Name;

use super::boundary::{GlobId, Lattice, ScopeId};
use super::entries::{Sight, Visibility};
use super::use_path::Reading;
use super::view::View;
use components::{Components, Settling};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Domain {
    Types,
    Macros,
}

impl Domain {
    const ALL: [Self; 2] = [Self::Types, Self::Macros];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key<'name> {
    domain: Domain,
    name: &'name Name,
}

impl<'name> Key<'name> {
    pub(super) const fn new(domain: Domain, name: &'name Name) -> Self {
        Key { domain, name }
    }

    pub(super) fn every(name: &'name Name) -> impl Iterator<Item = Key<'name>> {
        Domain::ALL.into_iter().map(move |domain| Key { domain, name })
    }
}

#[derive(Clone, Copy, Debug)]
enum Sought<'name> {
    Named(Key<'name>),
    Unlisted,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Keyed<'rows, Value> {
    key: Key<'rows>,
    value: Value,
}

impl<'rows, Value> Keyed<'rows, Value> {
    pub(super) const fn new(key: Key<'rows>, value: Value) -> Self {
        Keyed { key, value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Layer {
    Declared,
    Imported,
    Extern,
}

impl Layer {
    const ORDER: [Self; 3] = [Self::Declared, Self::Imported, Self::Extern];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Placed<'name> {
    key: Key<'name>,
    layer: Layer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Row<'rows> {
    layer: Layer,
    visibility: Visibility,
    view: View<'rows>,
}

impl<'rows> Row<'rows> {
    pub(super) const fn new(layer: Layer, visibility: Visibility, view: View<'rows>) -> Self {
        Row { layer, visibility, view }
    }

    fn seen(self, sight: Sight) -> View<'rows> {
        View::admitted(sight.admits(self.visibility), || self.view)
    }
}

#[derive(Debug, Default)]
pub(super) struct Explicit<'rows>(BTreeMap<Placed<'rows>, Vec<Row<'rows>>>);

impl<'rows> FromIterator<Keyed<'rows, Row<'rows>>> for Explicit<'rows> {
    fn from_iter<Rows: IntoIterator<Item = Keyed<'rows, Row<'rows>>>>(rows: Rows) -> Self {
        Explicit(rows.into_iter().fold(BTreeMap::new(), |mut placed: BTreeMap<Placed<'rows>, Vec<Row<'rows>>>, row| {
            placed.entry(Placed { key: row.key, layer: row.value.layer }).or_default().push(row.value);
            placed
        }))
    }
}

impl<'rows> Explicit<'rows> {
    fn found(&self, sought: Sought<'_>, sight: Sight) -> Option<View<'rows>> {
        match sought {
            Sought::Named(key) => self.named(key, sight),
            Sought::Unlisted => None,
        }
    }

    fn named(&self, key: Key<'_>, sight: Sight) -> Option<View<'rows>> {
        let rows = Layer::ORDER.iter().find_map(|layer| self.0.get(&Placed { key, layer: *layer }))?;
        Some(rows.iter().fold(View::ABSENT, |found, row| found.union(row.seen(sight))))
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Exposed<'rows> {
    default: View<'rows>,
    names: BTreeMap<Key<'rows>, View<'rows>>,
}

impl<'rows> FromIterator<Keyed<'rows, View<'rows>>> for Exposed<'rows> {
    fn from_iter<Views: IntoIterator<Item = Keyed<'rows, View<'rows>>>>(views: Views) -> Self {
        Exposed { default: View::ABSENT, names: views.into_iter().map(|keyed| (keyed.key, keyed.value)).collect() }
    }
}

impl<'rows> From<View<'rows>> for Exposed<'rows> {
    fn from(default: View<'rows>) -> Self {
        Exposed { default, names: BTreeMap::new() }
    }
}

impl<'rows> Exposed<'rows> {
    fn joined(&self, computed: &Self) -> Self {
        let keys: BTreeSet<Key<'rows>> = self.names.keys().chain(computed.names.keys()).copied().collect();
        let names = keys
            .into_iter()
            .map(|key| (key, self.found(Sought::Named(key)).union(computed.found(Sought::Named(key)))))
            .collect();
        Exposed { default: self.default.union(computed.default), names }
    }

    fn found(&self, sought: Sought<'_>) -> View<'rows> {
        match sought {
            Sought::Named(key) => self.names.get(&key).copied().unwrap_or(self.default),
            Sought::Unlisted => self.default,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct GlobView<'rows> {
    id: GlobId,
    visibility: Visibility,
    exposed: Exposed<'rows>,
}

impl<'rows> GlobView<'rows> {
    pub(super) const fn new(id: GlobId, visibility: Visibility, exposed: Exposed<'rows>) -> Self {
        GlobView { id, visibility, exposed }
    }

    fn found(&self, sought: Sought<'_>, sight: Sight) -> View<'rows> {
        View::admitted(sight.admits(self.visibility), || self.exposed.found(sought))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ModuleGlob<'rows> {
    glob: GlobView<'rows>,
    module: ScopeId,
    sight: Sight,
}

impl<'rows> ModuleGlob<'rows> {
    pub(super) const fn new(glob: GlobView<'rows>, module: ScopeId, sight: Sight) -> Self {
        ModuleGlob { glob, module, sight }
    }

    fn exposing(&self, names: Names<'_, 'rows>) -> Self {
        ModuleGlob {
            glob: GlobView {
                id: self.glob.id,
                visibility: self.glob.visibility,
                exposed: names.exposure(self.module, self.sight),
            },
            module: self.module,
            sight: self.sight,
        }
    }
}

impl Lattice for ModuleGlob<'_> {
    fn joined(&self, computed: Self) -> Self {
        ModuleGlob {
            glob: GlobView {
                id: self.glob.id,
                visibility: self.glob.visibility,
                exposed: self.glob.exposed.joined(&computed.glob.exposed),
            },
            module: self.module,
            sight: self.sight,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Lookup {
    sight: Sight,
    ignored: Option<GlobId>,
    reading: Reading,
}

impl Lookup {
    pub(super) const fn new(sight: Sight, ignored: Option<GlobId>, reading: Reading) -> Self {
        Self { sight, ignored, reading }
    }

    pub(super) const fn outer(self) -> Self {
        Self { sight: self.sight, ignored: None, reading: self.reading }
    }

    fn found<'rows>(self, level: Level<'_, 'rows>, sought: Sought<'_>) -> View<'rows> {
        let globbed = || {
            level
                .globs(self.ignored)
                .fold(View::ABSENT, |found, glob| found.union(self.reading.seen(glob.found(sought, self.sight))))
        };
        level.explicit.found(sought, self.sight).unwrap_or_else(globbed)
    }
}

#[derive(Clone, Copy, Debug)]
struct Level<'tables, 'rows> {
    explicit: &'tables Explicit<'rows>,
    fixed: &'tables [GlobView<'rows>],
    modules: &'tables [ModuleGlob<'rows>],
}

impl<'tables, 'rows> Level<'tables, 'rows> {
    fn exposure(self, sight: Sight) -> Exposed<'rows> {
        let explicit = self.explicit.0.keys().map(|placed| placed.key);
        let globbed = self.globs(None).flat_map(|glob| glob.exposed.names.keys().copied());
        let keys: BTreeSet<Key<'rows>> = explicit.chain(globbed).collect();
        let lookup = Lookup::new(sight, None, Reading::Written);
        let names = keys.into_iter().map(|key| (key, lookup.found(self, Sought::Named(key)))).collect();
        Exposed { default: lookup.found(self, Sought::Unlisted), names }
    }

    fn globs(self, ignored: Option<GlobId>) -> impl Iterator<Item = &'tables GlobView<'rows>> {
        self.fixed
            .iter()
            .chain(self.modules.iter().map(|module| &module.glob))
            .filter(move |glob| Some(glob.id) != ignored)
    }
}

#[derive(Clone, Copy, Debug)]
struct Names<'tables, 'rows> {
    explicit: &'tables [Explicit<'rows>],
    fixed: &'tables [Vec<GlobView<'rows>>],
    modules: &'tables [Vec<ModuleGlob<'rows>>],
}

impl<'tables, 'rows> Names<'tables, 'rows> {
    fn level(self, scope: ScopeId) -> Option<Level<'tables, 'rows>> {
        let rows = scope.row(self.explicit).zip(scope.row(self.fixed)).zip(scope.row(self.modules));
        rows.map(|((explicit, fixed), modules)| Level { explicit, fixed, modules })
    }

    fn found(self, scope: ScopeId, key: Key<'_>, lookup: Lookup) -> View<'rows> {
        match self.level(scope) {
            Some(level) => lookup.found(level, Sought::Named(key)),
            None => View::CONFLICT,
        }
    }

    fn exposure(self, scope: ScopeId, sight: Sight) -> Exposed<'rows> {
        match self.level(scope) {
            Some(level) => level.exposure(sight),
            None => Exposed::from(View::CONFLICT),
        }
    }

    fn exposed(self, scopes: &[ScopeId]) -> Vec<Exposing<'rows>> {
        let rows = scopes.iter().filter_map(|scope| scope.row(self.modules).map(|globs| (*scope, globs)));
        rows.map(|(scope, globs)| Exposing {
            scope,
            globs: globs.joined(globs.iter().map(|glob| glob.exposing(self)).collect()),
        })
        .collect()
    }
}

#[derive(Debug)]
struct Exposing<'rows> {
    scope: ScopeId,
    globs: Vec<ModuleGlob<'rows>>,
}

impl<'rows> Exposing<'rows> {
    fn written(self, mut modules: Vec<Vec<ModuleGlob<'rows>>>) -> Vec<Vec<ModuleGlob<'rows>>> {
        self.scope.row_mut(&mut modules).into_iter().zip([self.globs]).for_each(|(held, globs)| *held = globs);
        modules
    }
}

#[derive(Clone, Copy, Debug)]
struct Exposure<'tables, 'rows> {
    explicit: &'tables [Explicit<'rows>],
    fixed: &'tables [Vec<GlobView<'rows>>],
}

impl<'tables, 'rows> Exposure<'tables, 'rows> {
    const fn names<'held>(self, modules: &'held [Vec<ModuleGlob<'rows>>]) -> Names<'held, 'rows>
    where
        'tables: 'held,
    {
        Names { explicit: self.explicit, fixed: self.fixed, modules }
    }

    fn settled(self, modules: Vec<Vec<ModuleGlob<'rows>>>) -> Vec<Vec<ModuleGlob<'rows>>> {
        let successors: Vec<Vec<ScopeId>> =
            modules.iter().map(|globs| globs.iter().map(|glob| glob.module).collect()).collect();
        Components::ordered(&successors).settled(modules, &self)
    }

    fn advanced(
        modules: Vec<Vec<ModuleGlob<'rows>>>,
        exposed: Vec<Exposing<'rows>>,
        moved: Option<ScopeId>,
    ) -> ControlFlow<Vec<Vec<ModuleGlob<'rows>>>, Vec<Vec<ModuleGlob<'rows>>>> {
        match moved {
            Some(_) => {
                ControlFlow::Continue(exposed.into_iter().fold(modules, |modules, exposing| exposing.written(modules)))
            }
            None => ControlFlow::Break(modules),
        }
    }
}

impl<'rows> Settling<Vec<Vec<ModuleGlob<'rows>>>> for Exposure<'_, 'rows> {
    fn round(
        &self,
        modules: Vec<Vec<ModuleGlob<'rows>>>,
        scopes: &[ScopeId],
    ) -> ControlFlow<Vec<Vec<ModuleGlob<'rows>>>, Vec<Vec<ModuleGlob<'rows>>>> {
        let exposed = self.names(&modules).exposed(scopes);
        let moved = exposed
            .iter()
            .find(|exposing| exposing.scope.row(&modules) != Some(&exposing.globs))
            .map(|exposing| exposing.scope);
        Exposure::advanced(modules, exposed, moved)
    }
}

#[derive(Debug)]
pub(crate) struct Tables<'rows> {
    explicit: Vec<Explicit<'rows>>,
    fixed: Vec<Vec<GlobView<'rows>>>,
    modules: Vec<Vec<ModuleGlob<'rows>>>,
}

impl<'rows> Tables<'rows> {
    pub(super) fn new(
        explicit: Vec<Explicit<'rows>>,
        fixed: Vec<Vec<GlobView<'rows>>>,
        modules: Vec<Vec<ModuleGlob<'rows>>>,
    ) -> Self {
        let modules = Exposure { explicit: &explicit, fixed: &fixed }.settled(modules);
        Tables { explicit, fixed, modules }
    }

    pub(super) fn found(&self, scope: ScopeId, key: Key<'_>, lookup: Lookup) -> View<'rows> {
        let names = Names { explicit: &self.explicit, fixed: &self.fixed, modules: &self.modules };
        names.found(scope, key, lookup)
    }
}
