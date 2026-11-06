use std::path::Path;

use code_model::{ItemPath, Model, Named, Resolved, Site, Sited, Spelling, TypeRef, TypeUses};
use lint_core::Violation;
use lint_layout::{LineNumber, Name};

use crate::lexicon::{Identity, Lexicon, Lone};
use crate::scope::Scope;

#[derive(Debug)]
struct Candidate {
    identity: Identity,
    line: LineNumber,
}

#[derive(Debug)]
struct Signed<'model> {
    site: Site<'model>,
    found: Vec<Violation<'model>>,
}

#[derive(Debug)]
struct TypeWalk<'walk, 'model> {
    lexicon: &'walk Lexicon,
    scope: &'walk Scope<'model>,
    file: &'walk Path,
    found: Vec<Candidate>,
}

impl TypeWalk<'_, '_> {
    fn spelled(&mut self, named: &Named<Sited>, written: &ItemPath, line: LineNumber) {
        let resolved: &Resolved = named.as_ref();
        let solo = Lone::within(written);
        let identity = self.lexicon.identity(resolved, written);
        let direct = identity.and_then(|identity| identity.direct(solo));
        let target = solo.and(self.scope.local_alias(resolved, self.file)).and_then(TypeRef::named);
        let aliased = target.and_then(|target| self.aliased(target));
        self.found.extend(direct.into_iter().chain(aliased).map(|identity| Candidate { identity, line }));
        named.each_inner_use(self);
    }

    fn aliased(&self, target: &Named<Sited>) -> Option<Identity> {
        let written = AsRef::<Spelling>::as_ref(AsRef::<Sited>::as_ref(target)).written();
        written.and_then(|written| {
            self.lexicon.identity(target.as_ref(), written).and_then(|identity| identity.aliased(written))
        })
    }
}

impl TypeUses for TypeWalk<'_, '_> {
    fn named(&mut self, named: &Named<Sited>) {
        let spelling: &Spelling = AsRef::<Sited>::as_ref(named).as_ref();
        spelling.written().zip(spelling.line()).into_iter().fold(self, |walk, (written, line)| {
            walk.spelled(named, written, line);
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
            function.map(|function| self.signed(placed.site(), function.as_ref(), function.signature().written()))
        });
        let holdings = model.holdings().filter_map(|placed| {
            let holding = self.scope.holding(placed.entity());
            holding.map(|holding| self.signed(placed.site(), holding.as_ref(), [holding.as_ref()].into_iter()))
        });
        let mut signed: Vec<Signed<'model>> =
            functions.chain(holdings).filter(|signed| signed.found.first().is_some()).collect();
        signed.sort_by(|left, right| left.site.cmp(&right.site));
        signed.into_iter().flat_map(|signed| signed.found).collect()
    }

    fn signed<'ty>(
        &self,
        site: Site<'model>,
        context: &Name,
        types: impl Iterator<Item = &'ty TypeRef>,
    ) -> Signed<'model> {
        let mut walk = TypeWalk { lexicon: self.lexicon, scope: self.scope, file: site.path(), found: Vec::new() };
        types.for_each(|ty| ty.each_use(&mut walk));
        let found = walk
            .found
            .into_iter()
            .map(|found| Violation::new(Site::new(site.path(), found.line), found.identity.finding(context.clone())))
            .collect();
        Signed { site, found }
    }
}
