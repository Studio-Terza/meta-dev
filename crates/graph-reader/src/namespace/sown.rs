use std::collections::BTreeMap;
use std::iter;

use algebra::{Monoid, Semigroup};
use lint_layout::{Dependencies, Name};

use super::anchor::Anchors;
use super::boundary::ScopeId;
use super::entries::{Declared, Entries};
use super::scope::{CrateForm, ScopeKind, ScopeRow};
use super::{Namespace, Openings, Rooted, Top};

#[derive(Debug)]
struct ScopeShape {
    anchors: Anchors,
    chain: Vec<ScopeId>,
    ancestry: Vec<ScopeId>,
}

impl ScopeShape {
    fn root(scope: ScopeId, form: CrateForm) -> ScopeShape {
        ScopeShape { anchors: form.anchors(scope), chain: vec![scope], ancestry: vec![scope] }
    }
}

#[derive(Clone, Copy, Debug)]
struct Nesting<'parent> {
    parent: &'parent ScopeShape,
    kind: ScopeKind,
    scope: ScopeId,
}

impl From<Nesting<'_>> for ScopeShape {
    fn from(nesting: Nesting<'_>) -> ScopeShape {
        let Nesting { parent, kind, scope } = nesting;
        ScopeShape {
            anchors: kind.anchors(parent.anchors, scope),
            chain: kind.chain(&parent.chain, scope),
            ancestry: iter::once(scope).chain(parent.ancestry.iter().copied()).collect(),
        }
    }
}

#[derive(Debug)]
struct Declaration {
    scope: ScopeId,
    declared: Declared,
}

#[derive(Debug, Default)]
pub(crate) struct Sown<'read> {
    shapes: Vec<ScopeShape>,
    entries: Vec<Entries>,
    crates: BTreeMap<&'read Name, ScopeId>,
    dependencies: BTreeMap<ScopeId, &'read Dependencies>,
    declarations: Vec<Declaration>,
}

impl Semigroup for Sown<'_> {
    fn combine(self, other: Self) -> Self {
        let mut crates = self.crates;
        crates.extend(other.crates);
        let mut dependencies = self.dependencies;
        dependencies.extend(other.dependencies);
        Sown {
            shapes: self.shapes.combine(other.shapes),
            entries: self.entries.combine(other.entries),
            crates,
            dependencies,
            declarations: self.declarations.combine(other.declarations),
        }
    }
}

impl Monoid for Sown<'_> {}

impl<'read> Sown<'read> {
    pub(crate) fn declaring(scope: ScopeId, declared: Declared) -> Sown<'read> {
        Sown { declarations: vec![Declaration { scope, declared }], ..Sown::default() }
    }

    fn shaped(shape: Option<ScopeShape>, entries: Entries) -> Sown<'read> {
        let kept = shape.as_ref().map(|_| entries);
        Sown { shapes: shape.into_iter().collect(), entries: kept.into_iter().collect(), ..Sown::default() }
    }

    fn exported(&self, base: ScopeId) -> Vec<Declared> {
        let scopes = ScopeId::ids(&self.entries).map(|local| local.above(base));
        scopes.zip(&self.entries).flat_map(|(scope, entries)| entries.exported_aliases(scope)).collect()
    }
}

impl<'read> Rooted<'read> {
    pub(super) fn sown(self, base: ScopeId, entries: Entries) -> Sown<'read> {
        Sown {
            shapes: vec![ScopeShape::root(base, self.form)],
            entries: vec![entries],
            crates: self.name.map(|name| (name, base)).into_iter().collect(),
            dependencies: BTreeMap::from([(base, self.dependencies)]),
            declarations: Vec::new(),
        }
    }
}

impl<'read> From<Sown<'read>> for Namespace<'read> {
    fn from(sown: Sown<'read>) -> Namespace<'read> {
        let mut declared: BTreeMap<ScopeId, Vec<Declared>> = BTreeMap::new();
        sown.declarations.into_iter().for_each(|found| declared.entry(found.scope).or_default().push(found.declared));
        let entries: Vec<Entries> = ScopeId::ids(&sown.shapes)
            .zip(sown.entries)
            .map(|(scope, own)| {
                let later: Entries = declared.remove(&scope).into_iter().flatten().map(Entries::from).collect();
                own.combine(later)
            })
            .collect();
        let scopes = sown
            .shapes
            .into_iter()
            .zip(entries)
            .map(|(shape, held)| ScopeRow::new(shape.anchors, shape.chain, shape.ancestry, held))
            .collect();
        Namespace { scopes, crates: sown.crates, dependencies: sown.dependencies }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Sowing<'sown, 'read> {
    sown: &'sown Sown<'read>,
    base: ScopeId,
}

impl<'sown, 'read> From<&'sown Sown<'read>> for Sowing<'sown, 'read> {
    fn from(sown: &'sown Sown<'read>) -> Sowing<'sown, 'read> {
        Sowing { sown, base: ScopeId::after(&sown.shapes) }
    }
}

impl<'sown, 'read> Sowing<'sown, 'read> {
    pub(crate) fn base(self) -> ScopeId {
        self.base
    }

    pub(crate) fn placed(self, top: Top<'read>, openings: Openings) -> Sown<'read> {
        let base = self.base;
        let root = top.root(self);
        let topped = top.sown(self, openings.top.above(base));
        let placed = openings.nested.into_iter().fold(topped, |placed, opening| {
            let scope = ScopeId::after(&placed.shapes).above(base);
            let shape = self
                .shape(&placed, opening.parent.above(base))
                .map(|parent| ScopeShape::from(Nesting { parent, kind: opening.kind, scope }));
            placed.combine(Sown::shaped(shape, opening.entries.above(base)))
        });
        let exported = placed.exported(base);
        let aliased = root.map_or_default(|scope| {
            Monoid::concat(exported.into_iter().map(|declared| Sown::declaring(scope, declared)))
        });
        placed.combine(aliased)
    }

    pub(super) fn anchors(self, parent: ScopeId) -> Anchors {
        parent.row(&self.sown.shapes).map_or_default(|found| found.anchors)
    }

    pub(super) fn child(self, parent: ScopeId, declared: Declared, entries: Entries) -> Sown<'read> {
        let nesting = |row| Nesting { parent: row, kind: ScopeKind::Module, scope: self.base };
        let opened = parent.row(&self.sown.shapes).map(|row| ScopeShape::from(nesting(row)));
        Sown::shaped(opened, entries).combine(Sown::declaring(parent, declared))
    }

    fn shape<'found>(self, local: &'found Sown<'read>, scope: ScopeId) -> Option<&'found ScopeShape>
    where
        'sown: 'found,
    {
        scope.row(&self.sown.shapes).or_else(|| scope.row_after(self.base, &local.shapes))
    }
}
