use std::collections::BTreeSet;

use graph::ItemPath;
use lint_layout::Name;

use super::binding::Binding;
use algebra::Semigroup;

use super::boundary::{GlobId, Lattice, Pointwise, ScopeId};
use super::entries::{Sight, Visibility};
use super::resolver::{Resolution, Resolver};
use super::rules::{RulesDefinition, Written};
use super::std_names::{Listed, StdNames};
use super::tables::{Declaring, Exposed, GlobView, Key, Keyed, Layer, ModuleGlob, Row, Tables};
use super::view::View;
use super::{Namespace, ScopeRow};
use crate::parallel::Parallel;

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
    Scoped(Vec<Name>),
    Opaque,
}

impl Target {
    pub(super) fn listed(module: &ItemPath) -> Target {
        match StdNames::listed(module) {
            Some(listed) => Target::Listed(listed),
            None => Target::Opaque,
        }
    }

    pub(super) fn narrowed(rules: Option<RulesDefinition>, written: &Written) -> Target {
        match rules {
            Some(_) => Target::scoped(written.names()),
            None => Target::Opaque,
        }
    }

    fn scoped(names: &BTreeSet<Name>) -> Target {
        Target::Scoped(names.iter().cloned().collect())
    }

    pub(super) fn fixed(&self, id: GlobId, visibility: Visibility) -> Option<GlobView<'_>> {
        match self {
            Target::Waiting => Some(GlobView::new(id, visibility, Exposed::from(View::UNEXPANDED))),
            Target::Module(..) => None,
            Target::Listed(listed) => {
                Some(GlobView::new(id, visibility, listed.iter().flat_map(Listed::views).collect()))
            }
            Target::Opaque => Some(GlobView::new(id, visibility, Exposed::from(View::CONFLICT))),
            Target::Scoped(names) => Some(GlobView::new(id, visibility, Target::shadowed(names))),
        }
    }

    pub(super) fn module(&self, id: GlobId, visibility: Visibility) -> Option<ModuleGlob<'_>> {
        match self {
            Target::Module(module, sight) => {
                Some(ModuleGlob::new(GlobView::new(id, visibility, Exposed::default()), *module, *sight))
            }
            Target::Waiting | Target::Listed(_) | Target::Scoped(_) | Target::Opaque => None,
        }
    }

    pub(super) fn declaring(&self) -> Declaring<'_> {
        match self {
            Target::Scoped(names) => Declaring::Only(names),
            Target::Waiting | Target::Module(..) | Target::Listed(_) | Target::Opaque => Declaring::Anything,
        }
    }

    fn shadowed(names: &[Name]) -> Exposed<'_> {
        names.iter().flat_map(Key::every).map(|key| Keyed::new(key, View::CONFLICT)).collect()
    }

    fn forced(self) -> Target {
        match self {
            Target::Waiting | Target::Opaque => Target::Opaque,
            Target::Module(module, sight) => Target::Module(module, sight),
            Target::Listed(listed) => Target::Listed(listed),
            Target::Scoped(names) => Target::Scoped(names),
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
            imports: self
                .imports
                .into_iter()
                .map(|import| Resolution::Bound(Option::from(import).unwrap_or(Binding::UNBOUND)))
                .collect(),
            targets: self.targets.into_iter().map(Target::forced).collect(),
        }
    }
}

impl Semigroup for Target {
    fn combine(self, computed: Target) -> Target {
        match self {
            Target::Waiting => computed,
            Target::Module(..) | Target::Listed(_) | Target::Scoped(_) | Target::Opaque => {
                Some(self).filter(|held| [Target::Waiting, held.clone()].contains(&computed)).unwrap_or(Target::Opaque)
            }
        }
    }
}

impl Semigroup for ScopeSettling {
    fn combine(self, computed: ScopeSettling) -> ScopeSettling {
        let imports = Pointwise::from(self.imports).combine(Pointwise::from(computed.imports));
        let targets = Pointwise::from(self.targets).combine(Pointwise::from(computed.targets));
        ScopeSettling { imports: imports.into(), targets: targets.into() }
    }
}

impl Seeds<'_> {
    pub(crate) fn settled(&self) -> Vec<ScopeSettling> {
        let unsettled: Vec<ScopeSettling> = self.space.scopes.iter().map(|row| row.entries().unsettled()).collect();
        let settled = Pointwise::from(unsettled).settled(|settling| Pointwise::from(self.advanced(settling.as_ref())));
        Vec::from(settled).into_iter().map(ScopeSettling::forced).collect()
    }

    pub(crate) fn tables<'rows>(&'rows self, settling: &'rows [ScopeSettling]) -> Tables<'rows> {
        let scopes = &self.space.scopes;
        let ids: Vec<ScopeId> = ScopeId::ids(scopes).collect();
        let explicit = Parallel::available().mapped(&ids, |scope| {
            let row = scope.row(scopes).map(ScopeRow::entries);
            let seeds = scope.row(&self.scopes).map_or(&[][..], Vec::as_slice);
            let imports = scope.row(settling).map_or(&[][..], |settled| settled.imports.as_slice());
            row.map_or_default(|entries| entries.explicit(seeds, imports))
        });
        let fixed = scopes.iter().zip(settling).map(|(row, settled)| row.entries().fixed(&settled.targets));
        let modules = scopes.iter().zip(settling).map(|(row, settled)| row.entries().modules(&settled.targets));
        let declaring = scopes.iter().zip(settling).map(|(row, settled)| row.entries().declaring(&settled.targets));
        Tables::new(explicit, fixed.collect(), modules.collect(), declaring.collect())
    }

    fn advanced(&self, settling: &[ScopeSettling]) -> Vec<ScopeSettling> {
        let tables = self.tables(settling);
        let resolver = Resolver::new(self.space, &tables);
        let scopes = &self.space.scopes;
        let ids: Vec<ScopeId> = ScopeId::ids(scopes).collect();
        Parallel::available().mapped(&ids, |scope| {
            scope.row(scopes).map_or_else(
                || ScopeSettling::new(Vec::new(), Vec::new()),
                |row| row.entries().settled(*scope, &resolver),
            )
        })
    }
}
