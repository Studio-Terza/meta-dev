use std::path::Path;

use graph::{ItemPath, Model, Named, Resolved, Site, Sited, Spelling, TypeRef, TypeUses};
use lint_core::Violation;
use lint_layout::{Name, Position};

use crate::lexicon::{Identity, Lexicon, Lone};
use crate::scope::Scope;

#[derive(Debug)]
struct Candidate {
    identity: Identity,
    position: Position,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Spot<'model> {
    path: &'model Path,
    position: Position,
}

#[derive(Debug)]
struct Ranked<'model> {
    spot: Spot<'model>,
    violation: Violation<'model>,
}

#[derive(Debug)]
struct TypeWalk<'walk, 'model> {
    lexicon: &'walk Lexicon,
    scope: &'walk Scope<'model>,
    file: &'walk Path,
    found: Vec<Candidate>,
}

impl TypeWalk<'_, '_> {
    fn spelled(&mut self, named: &Named<Sited>, written: &ItemPath, position: Position) {
        let resolved: &Resolved = named.as_ref();
        let solo = Lone::within(written);
        let identity = self.lexicon.identity(resolved, written);
        let direct = identity.and_then(|known| known.direct(solo));
        let target = solo.and(self.scope.local_alias(resolved, self.file)).and_then(TypeRef::named);
        let aliased = target.and_then(|local| self.aliased(local));
        self.found.extend(direct.into_iter().chain(aliased).map(|found| Candidate { identity: found, position }));
        named.each_inner_use(self);
    }

    fn aliased(&self, target: &Named<Sited>) -> Option<Identity> {
        let written = AsRef::<Spelling>::as_ref(AsRef::<Sited>::as_ref(target)).written();
        written
            .and_then(|path| self.lexicon.identity(target.as_ref(), path).and_then(|identity| identity.aliased(path)))
    }
}

impl TypeUses for TypeWalk<'_, '_> {
    fn named(&mut self, named: &Named<Sited>) {
        let spelling: &Spelling = AsRef::<Sited>::as_ref(named).as_ref();
        spelling.written().zip(spelling.position()).into_iter().fold(self, |walk, (written, position)| {
            walk.spelled(named, written, position);
            walk
        });
    }
}

#[derive(Debug)]
pub struct Signatures<'read, 'model> {
    lexicon: &'read Lexicon,
    scope: &'read Scope<'model>,
}

impl<'read, 'model> Signatures<'read, 'model> {
    pub fn new(lexicon: &'read Lexicon, scope: &'read Scope<'model>) -> Signatures<'read, 'model> {
        Signatures { lexicon, scope }
    }

    pub fn found(&self, model: &'model Model) -> Vec<Violation<'model>> {
        let functions = model.functions().filter_map(|placed| {
            let function = self.scope.function(placed.entity());
            function.map(|found| self.ranked(placed.site(), found.as_ref(), found.signature().written()))
        });
        let holdings = model.holdings().filter_map(|placed| {
            let holding = self.scope.holding(placed.entity());
            holding.map(|held| self.ranked(placed.site(), held.as_ref(), [held.as_ref()].into_iter()))
        });
        let mut ranked: Vec<Ranked<'model>> = functions.chain(holdings).flatten().collect();
        ranked.sort_by_key(|entry| entry.spot);
        ranked.into_iter().map(|entry| entry.violation).collect()
    }

    fn ranked<'ty>(
        &self,
        site: Site<'model>,
        context: &Name,
        types: impl Iterator<Item = &'ty TypeRef>,
    ) -> Vec<Ranked<'model>> {
        let mut walk = TypeWalk { lexicon: self.lexicon, scope: self.scope, file: site.path(), found: Vec::new() };
        types.for_each(|ty| ty.each_use(&mut walk));
        walk.found
            .into_iter()
            .map(|found| Ranked {
                spot: Spot { path: site.path(), position: found.position },
                violation: Violation::new(
                    Site::new(site.path(), found.position.line()),
                    found.identity.finding(context.clone()),
                ),
            })
            .collect()
    }
}
