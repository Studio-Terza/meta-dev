use code_model::ItemPath;

use super::Namespace;
use super::binding::Binding;
use super::boundary::{GlobId, Lattice, ScopeId};
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    fn kept(self, held: &Target) -> Target {
        match self {
            Target::Waiting => held.clone(),
            Target::Module(..) | Target::Listed(_) | Target::Opaque => {
                [self].into_iter().find(|found| found == held).unwrap_or(Target::Opaque)
            }
        }
    }

    fn forced(self) -> Target {
        match self {
            Target::Waiting | Target::Opaque => Target::Opaque,
            Target::Module(module, sight) => Target::Module(module, sight),
            Target::Listed(listed) => Target::Listed(listed),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ScopeSettling {
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
}

impl Lattice for Target {
    fn joined(&self, computed: Target) -> Target {
        match self {
            Target::Waiting => computed,
            Target::Module(..) | Target::Listed(_) | Target::Opaque => computed.kept(self),
        }
    }
}

impl Lattice for ScopeSettling {
    fn joined(&self, computed: ScopeSettling) -> ScopeSettling {
        ScopeSettling { imports: self.imports.joined(computed.imports), targets: self.targets.joined(computed.targets) }
    }
}

impl Seeds<'_> {
    pub(crate) fn settled(&self) -> Vec<ScopeSettling> {
        let unsettled: Vec<ScopeSettling> = self.space.scopes.iter().map(|row| row.entries().unsettled()).collect();
        unsettled.settled(|settling| self.advanced(settling)).into_iter().map(ScopeSettling::forced).collect()
    }

    pub(crate) fn tables<'rows>(&'rows self, settling: &'rows [ScopeSettling]) -> Tables<'rows> {
        let scopes = &self.space.scopes;
        let explicit = scopes
            .iter()
            .zip(&self.scopes)
            .zip(settling)
            .map(|((row, seeds), settled)| row.entries().explicit(seeds, &settled.imports));
        let fixed = scopes.iter().zip(settling).map(|(row, settled)| row.entries().fixed(&settled.targets));
        let modules = scopes.iter().zip(settling).map(|(row, settled)| row.entries().modules(&settled.targets));
        Tables::new(explicit.collect(), fixed.collect(), modules.collect())
    }

    fn advanced(&self, settling: &[ScopeSettling]) -> Vec<ScopeSettling> {
        let tables = self.tables(settling);
        let resolver = Resolver::new(self.space, &tables);
        let scopes = &self.space.scopes;
        ScopeId::ids(scopes).zip(scopes).map(|(scope, row)| row.entries().settled(scope, &resolver)).collect()
    }
}
