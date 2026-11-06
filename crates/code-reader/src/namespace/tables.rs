use std::collections::BTreeMap;

use lint_layout::Name;

use super::boundary::{GlobId, Measure, ScopeId};
use super::converging::Converging;
use super::entries::{Sight, Visibility};
use super::view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Domain {
    Types,
    Macros,
}

impl Domain {
    const ALL: [Domain; 2] = [Domain::Types, Domain::Macros];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Key<'name> {
    domain: Domain,
    name: &'name Name,
}

impl<'name> Key<'name> {
    pub(super) fn new(domain: Domain, name: &'name Name) -> Key<'name> {
        Key { domain, name }
    }

    pub(super) fn every(name: &'name Name) -> impl Iterator<Item = Key<'name>> {
        Domain::ALL.into_iter().map(move |domain| Key { domain, name })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Keyed<'rows, Value> {
    key: Key<'rows>,
    value: Value,
}

impl<'rows, Value> Keyed<'rows, Value> {
    pub(super) fn new(key: Key<'rows>, value: Value) -> Keyed<'rows, Value> {
        Keyed { key, value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Layer {
    Declared,
    Imported,
    Extern,
}

impl Layer {
    const ORDER: [Layer; 3] = [Layer::Declared, Layer::Imported, Layer::Extern];

    fn found<'rows>(self, rows: &[Row<'rows>], sight: Sight) -> View<'rows> {
        rows.iter().filter(|row| row.layer == self).fold(View::ABSENT, |found, row| found.union(row.seen(sight)))
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Row<'rows> {
    layer: Layer,
    visibility: Visibility,
    view: View<'rows>,
}

impl<'rows> Row<'rows> {
    pub(super) fn new(layer: Layer, visibility: Visibility, view: View<'rows>) -> Row<'rows> {
        Row { layer, visibility, view }
    }

    fn seen(self, sight: Sight) -> View<'rows> {
        View::admitted(sight.admits(self.visibility), || self.view)
    }
}

#[derive(Debug, Default)]
pub(super) struct Explicit<'rows>(BTreeMap<Key<'rows>, Vec<Row<'rows>>>);

impl<'rows> FromIterator<Keyed<'rows, Row<'rows>>> for Explicit<'rows> {
    fn from_iter<Rows: IntoIterator<Item = Keyed<'rows, Row<'rows>>>>(rows: Rows) -> Explicit<'rows> {
        Explicit(rows.into_iter().fold(BTreeMap::new(), |mut keyed: BTreeMap<Key<'rows>, Vec<Row<'rows>>>, row| {
            keyed.entry(row.key).or_default().push(row.value);
            keyed
        }))
    }
}

impl<'rows> Explicit<'rows> {
    fn found(&self, key: &Key<'_>, sight: Sight) -> View<'rows> {
        let rows = self.0.get(key).map(Vec::as_slice).unwrap_or_default();
        Layer::ORDER.iter().fold(View::ABSENT, |found, layer| found.or_else(|| layer.found(rows, sight)))
    }
}

#[derive(Debug, Default)]
pub(super) struct Exposed<'rows> {
    default: View<'rows>,
    names: BTreeMap<Key<'rows>, View<'rows>>,
}

impl<'rows> FromIterator<Keyed<'rows, View<'rows>>> for Exposed<'rows> {
    fn from_iter<Views: IntoIterator<Item = Keyed<'rows, View<'rows>>>>(views: Views) -> Exposed<'rows> {
        Exposed { default: View::ABSENT, names: views.into_iter().map(|keyed| (keyed.key, keyed.value)).collect() }
    }
}

impl<'rows> From<View<'rows>> for Exposed<'rows> {
    fn from(default: View<'rows>) -> Exposed<'rows> {
        Exposed { default, names: BTreeMap::new() }
    }
}

impl<'rows> Exposed<'rows> {
    fn found(&self, key: &Key<'_>) -> View<'rows> {
        self.names.get(key).copied().unwrap_or(self.default)
    }

    fn keyed(&self, keys: Vec<Key<'rows>>) -> Exposed<'rows> {
        let names = keys.into_iter().chain(self.names.keys().copied()).map(|key| (key, View::ABSENT)).collect();
        Exposed { default: self.default, names }
    }

    fn valued(&self, default: View<'rows>, value: impl Fn(&Key<'rows>) -> View<'rows>) -> Exposed<'rows> {
        Exposed { default, names: self.names.keys().map(|key| (*key, value(key))).collect() }
    }

    fn height(&self) -> Measure {
        self.names.values().chain([&self.default]).map(|view| view.height()).sum()
    }
}

#[derive(Debug)]
pub(super) struct GlobView<'rows> {
    id: GlobId,
    visibility: Visibility,
    exposed: Exposed<'rows>,
}

impl<'rows> GlobView<'rows> {
    pub(super) fn new(id: GlobId, visibility: Visibility, exposed: Exposed<'rows>) -> GlobView<'rows> {
        GlobView { id, visibility, exposed }
    }

    fn found(&self, key: &Key<'_>, sight: Sight) -> View<'rows> {
        View::admitted(sight.admits(self.visibility), || self.exposed.found(key))
    }

    fn defaulted(&self, sight: Sight) -> View<'rows> {
        View::admitted(sight.admits(self.visibility), || self.exposed.default)
    }

    fn keys(&self) -> impl Iterator<Item = Key<'rows>> + '_ {
        self.exposed.names.keys().copied()
    }
}

#[derive(Debug)]
pub(super) struct ModuleGlob<'rows> {
    glob: GlobView<'rows>,
    module: ScopeId,
    sight: Sight,
}

impl<'rows> ModuleGlob<'rows> {
    pub(super) fn new(glob: GlobView<'rows>, module: ScopeId, sight: Sight) -> ModuleGlob<'rows> {
        ModuleGlob { glob, module, sight }
    }

    fn keyed(&self, keys: Vec<Key<'rows>>) -> ModuleGlob<'rows> {
        let exposed = self.glob.exposed.keyed(keys);
        self.exposing(exposed)
    }

    fn valued(&self, names: Names<'_, 'rows>) -> ModuleGlob<'rows> {
        let default = names.defaulted(self.module, self.sight);
        let exposed = self.glob.exposed.valued(default, |key| names.found(self.module, key, self.sight, None));
        self.exposing(exposed)
    }

    fn exposing(&self, exposed: Exposed<'rows>) -> ModuleGlob<'rows> {
        ModuleGlob {
            glob: GlobView { id: self.glob.id, visibility: self.glob.visibility, exposed },
            module: self.module,
            sight: self.sight,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Level<'tables, 'rows> {
    explicit: &'tables Explicit<'rows>,
    fixed: &'tables [GlobView<'rows>],
    modules: &'tables [ModuleGlob<'rows>],
}

impl<'tables, 'rows> Level<'tables, 'rows> {
    fn found(self, key: &Key<'_>, sight: Sight, ignored: Option<GlobId>) -> View<'rows> {
        let globbed = || self.globs(ignored).fold(View::ABSENT, |found, glob| found.union(glob.found(key, sight)));
        self.explicit.found(key, sight).or_else(globbed)
    }

    fn defaulted(self, sight: Sight) -> View<'rows> {
        self.globs(None).fold(View::ABSENT, |found, glob| found.union(glob.defaulted(sight)))
    }

    fn keys(self) -> Vec<Key<'rows>> {
        self.explicit.0.keys().copied().chain(self.globs(None).flat_map(GlobView::keys)).collect()
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

    fn found(self, scope: ScopeId, key: &Key<'_>, sight: Sight, ignored: Option<GlobId>) -> View<'rows> {
        match self.level(scope) {
            Some(level) => level.found(key, sight, ignored),
            None => View::CONFLICT,
        }
    }

    fn defaulted(self, scope: ScopeId, sight: Sight) -> View<'rows> {
        match self.level(scope) {
            Some(level) => level.defaulted(sight),
            None => View::CONFLICT,
        }
    }

    fn keys(self, scope: ScopeId) -> Vec<Key<'rows>> {
        self.level(scope).map(Level::keys).unwrap_or_default()
    }
}

#[derive(Debug)]
struct Spread<'tables, 'rows> {
    explicit: &'tables [Explicit<'rows>],
    fixed: &'tables [Vec<GlobView<'rows>>],
    modules: Vec<Vec<ModuleGlob<'rows>>>,
}

impl<'tables, 'rows> Spread<'tables, 'rows> {
    fn names(&self) -> Names<'_, 'rows> {
        Names { explicit: self.explicit, fixed: self.fixed, modules: &self.modules }
    }

    fn stepped(
        &self,
        step: impl Fn(&ModuleGlob<'rows>, Names<'_, 'rows>) -> ModuleGlob<'rows>,
    ) -> Spread<'tables, 'rows> {
        let names = self.names();
        let modules = self.modules.iter().map(|globs| globs.iter().map(|glob| step(glob, names)).collect()).collect();
        Spread { explicit: self.explicit, fixed: self.fixed, modules }
    }

    fn measured(&self, measure: impl Fn(&Exposed<'rows>) -> Measure) -> Measure {
        self.modules.iter().flatten().map(|glob| measure(&glob.glob.exposed)).sum()
    }
}

#[derive(Debug)]
struct Keying<'tables, 'rows>(Spread<'tables, 'rows>);

impl Converging for Keying<'_, '_> {
    fn measure(&self) -> Measure {
        self.0.measured(|exposed| Measure::from(exposed.names.len()))
    }

    fn advanced(&self) -> Self {
        Keying(self.0.stepped(|glob, names| glob.keyed(names.keys(glob.module))))
    }
}

#[derive(Debug)]
struct Valuing<'tables, 'rows>(Spread<'tables, 'rows>);

impl Converging for Valuing<'_, '_> {
    fn measure(&self) -> Measure {
        self.0.measured(Exposed::height)
    }

    fn advanced(&self) -> Self {
        Valuing(self.0.stepped(ModuleGlob::valued))
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
    ) -> Tables<'rows> {
        let keyed = Keying(Spread { explicit: &explicit, fixed: &fixed, modules }).converged().0.modules;
        let valued = Valuing(Spread { explicit: &explicit, fixed: &fixed, modules: keyed }).converged().0.modules;
        Tables { explicit, fixed, modules: valued }
    }

    pub(super) fn found(&self, scope: ScopeId, key: &Key<'_>, sight: Sight, ignored: Option<GlobId>) -> View<'rows> {
        Names { explicit: &self.explicit, fixed: &self.fixed, modules: &self.modules }.found(scope, key, sight, ignored)
    }
}
