use code_model::ItemPath;

use super::Namespace;
use super::binding::Binding;
use super::boundary::{GlobId, Measure, ScopeId};
use super::converging::Converging;
use super::entries::{Sight, Visibility};
use super::resolver::{Resolution, Resolver};
use super::std_names::{Listed, StdNames};
use super::tables::{Exposed, GlobView, Key, Keyed, Layer, ModuleGlob, Row, Tables};
use super::view::View;

#[derive(Debug)]
pub(super) struct Seed<'space> {
    key: Key<'space>,
    layer: Layer,
    visibility: Visibility,
    binding: Binding,
}

impl<'space> Seed<'space> {
    pub(super) fn new(key: Key<'space>, layer: Layer, visibility: Visibility, binding: Binding) -> Seed<'space> {
        Seed { key, layer, visibility, binding }
    }

    pub(super) fn row(&self) -> Keyed<'_, Row<'_>> {
        Keyed::new(self.key, Row::new(self.layer, self.visibility, View::from(&self.binding)))
    }
}

#[derive(Debug)]
pub(crate) struct Seeds<'space> {
    space: &'space Namespace<'space>,
    scopes: Vec<Vec<Seed<'space>>>,
}

impl<'space> From<&'space Namespace<'space>> for Seeds<'space> {
    fn from(space: &'space Namespace<'space>) -> Seeds<'space> {
        let scopes =
            ScopeId::ids(&space.scopes).zip(&space.scopes).map(|(scope, row)| row.entries().seeds(scope, space));
        Seeds { space, scopes: scopes.collect() }
    }
}

#[derive(Debug)]
pub(super) enum Target {
    Waiting,
    Module(ScopeId, Sight),
    Listed(Vec<Listed>),
    Opaque,
}

impl Target {
    pub(super) fn listed(module: &ItemPath) -> Target {
        match StdNames::listed(module) {
            Some(listed) => Target::Listed(listed),
            None => Target::Opaque,
        }
    }

    pub(super) fn fixed(&self, id: GlobId, visibility: Visibility) -> Option<GlobView<'_>> {
        match self {
            Target::Waiting => Some(GlobView::new(id, visibility, Exposed::from(View::UNEXPANDED))),
            Target::Module(..) => None,
            Target::Listed(listed) => {
                Some(GlobView::new(id, visibility, listed.iter().flat_map(Listed::views).collect()))
            }
            Target::Opaque => Some(GlobView::new(id, visibility, Exposed::from(View::CONFLICT))),
        }
    }

    pub(super) fn module(&self, id: GlobId, visibility: Visibility) -> Option<ModuleGlob<'_>> {
        match self {
            Target::Module(module, sight) => {
                Some(ModuleGlob::new(GlobView::new(id, visibility, Exposed::default()), *module, *sight))
            }
            Target::Waiting | Target::Listed(_) | Target::Opaque => None,
        }
    }

    fn forced(self) -> Target {
        match self {
            Target::Waiting | Target::Opaque => Target::Opaque,
            Target::Module(module, sight) => Target::Module(module, sight),
            Target::Listed(listed) => Target::Listed(listed),
        }
    }

    fn waiting(&self) -> Measure {
        match self {
            Target::Waiting => Measure::ONE,
            Target::Module(..) | Target::Listed(_) | Target::Opaque => Measure::ZERO,
        }
    }
}

#[derive(Debug)]
pub(super) struct ScopeSettling {
    imports: Vec<Resolution>,
    targets: Vec<Target>,
}

impl ScopeSettling {
    pub(super) fn new(imports: Vec<Resolution>, targets: Vec<Target>) -> ScopeSettling {
        ScopeSettling { imports, targets }
    }

    fn forced(self) -> ScopeSettling {
        ScopeSettling {
            imports: self.imports.into_iter().map(|import| Resolution::Bound(import.forced())).collect(),
            targets: self.targets.into_iter().map(Target::forced).collect(),
        }
    }

    fn waiting(&self) -> Measure {
        self.imports.iter().map(Resolution::waiting).chain(self.targets.iter().map(Target::waiting)).sum()
    }
}

#[derive(Debug)]
pub(crate) struct Settled<'seeds, 'space> {
    seeds: &'seeds Seeds<'space>,
    settling: Vec<ScopeSettling>,
}

impl<'seeds, 'space> From<&'seeds Seeds<'space>> for Settled<'seeds, 'space> {
    fn from(seeds: &'seeds Seeds<'space>) -> Settled<'seeds, 'space> {
        let settling = seeds.space.scopes.iter().map(|row| row.entries().unsettled()).collect();
        let settled = Settled { seeds, settling }.converged();
        Settled { seeds, settling: settled.settling.into_iter().map(ScopeSettling::forced).collect() }
    }
}

impl Converging for Settled<'_, '_> {
    fn measure(&self) -> Measure {
        self.settling.iter().map(ScopeSettling::waiting).sum()
    }

    fn advanced(&self) -> Self {
        let tables = self.tables();
        let resolver = Resolver::new(self.seeds.space, &tables);
        let scopes = &self.seeds.space.scopes;
        let settling = ScopeId::ids(scopes).zip(scopes).map(|(scope, row)| row.entries().settled(scope, &resolver));
        Settled { seeds: self.seeds, settling: settling.collect() }
    }
}

impl Settled<'_, '_> {
    pub(crate) fn tables(&self) -> Tables<'_> {
        let scopes = &self.seeds.space.scopes;
        let explicit = scopes
            .iter()
            .zip(&self.seeds.scopes)
            .zip(&self.settling)
            .map(|((row, seeds), settled)| row.entries().explicit(seeds, &settled.imports));
        let fixed = scopes.iter().zip(&self.settling).map(|(row, settled)| row.entries().fixed(&settled.targets));
        let modules = scopes.iter().zip(&self.settling).map(|(row, settled)| row.entries().modules(&settled.targets));
        Tables::new(explicit.collect(), fixed.collect(), modules.collect())
    }
}
