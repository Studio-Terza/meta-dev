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
    pub(super) fn new(key: Key<'space>, layer: Layer, visibility: Visibility, binding: Binding) -> Self {
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
    fn from(space: &'space Namespace<'space>) -> Self {
        let scopes =
            ScopeId::ids(&space.scopes).zip(&space.scopes).map(|(scope, row)| row.entries().seeds(scope, space));
        Seeds { space, scopes: scopes.collect() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Waiting,
    Module(ScopeId, Sight),
    Listed(Vec<Listed>),
    Opaque,
}

impl Target {
    pub(super) fn listed(module: &ItemPath) -> Self {
        match StdNames::listed(module) {
            Some(listed) => Self::Listed(listed),
            None => Self::Opaque,
        }
    }

    pub(super) fn fixed(&self, id: GlobId, visibility: Visibility) -> Option<GlobView<'_>> {
        match self {
            Self::Waiting => Some(GlobView::new(id, visibility, Exposed::from(View::UNEXPANDED))),
            Self::Module(..) => None,
            Self::Listed(listed) => {
                Some(GlobView::new(id, visibility, listed.iter().flat_map(Listed::views).collect()))
            }
            Self::Opaque => Some(GlobView::new(id, visibility, Exposed::from(View::CONFLICT))),
        }
    }

    pub(super) fn module(&self, id: GlobId, visibility: Visibility) -> Option<ModuleGlob<'_>> {
        match self {
            Self::Module(module, sight) => {
                Some(ModuleGlob::new(GlobView::new(id, visibility, Exposed::default()), *module, *sight))
            }
            Self::Waiting | Self::Listed(_) | Self::Opaque => None,
        }
    }

    fn kept(self, held: &Self) -> Self {
        match self {
            Self::Waiting => held.clone(),
            Self::Module(..) | Self::Listed(_) | Self::Opaque => {
                [self].into_iter().find(|found| found == held).unwrap_or(Self::Opaque)
            }
        }
    }

    fn forced(self) -> Self {
        match self {
            Self::Waiting | Self::Opaque => Self::Opaque,
            Self::Module(module, sight) => Self::Module(module, sight),
            Self::Listed(listed) => Self::Listed(listed),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ScopeSettling {
    imports: Vec<Resolution>,
    targets: Vec<Target>,
}

impl ScopeSettling {
    pub(super) fn new(imports: Vec<Resolution>, targets: Vec<Target>) -> Self {
        Self { imports, targets }
    }

    fn forced(self) -> Self {
        Self {
            imports: self.imports.into_iter().map(|import| Resolution::Bound(import.forced())).collect(),
            targets: self.targets.into_iter().map(Target::forced).collect(),
        }
    }
}

impl Lattice for Target {
    fn joined(&self, computed: Self) -> Self {
        match self {
            Self::Waiting => computed,
            Self::Module(..) | Self::Listed(_) | Self::Opaque => computed.kept(self),
        }
    }
}

impl Lattice for ScopeSettling {
    fn joined(&self, computed: Self) -> Self {
        Self { imports: self.imports.joined(computed.imports), targets: self.targets.joined(computed.targets) }
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
