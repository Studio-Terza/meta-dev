mod place;
mod reach;
mod stand_ins;
mod target;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use lint_core::{Site, SourceFile};
use lint_item_graph::{Breadth, ReferencedNames, Scope, Traversal};
use lint_layout::{Name, UsePath};
use lint_placement::CrateName;
use transition_facts::{Endpoint, Nature, Transition};
use type_facts::{Dependency, Implementation};

use super::link::Link;
use super::member::Member;
use place::Place;
use reach::Reach;
pub(crate) use stand_ins::StandIns;
use target::Target;

#[derive(Debug, Default)]
pub(crate) struct Resolver<'found> {
    declaring: BTreeMap<&'found Name, Vec<Member<'found>>>,
    modules: BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
    places: BTreeMap<&'found Path, Place<'found>>,
    stand_ins: StandIns<'found>,
}

impl<'found> Resolver<'found> {
    pub(crate) fn of_members(
        members: impl Iterator<Item = Member<'found>>,
        files: &[&'found SourceFile],
        stand_ins: StandIns<'found>,
    ) -> Resolver<'found> {
        let modules = files.iter().fold(BTreeMap::new(), |mut modules: BTreeMap<_, BTreeSet<_>>, file| {
            let placement = file.placement();
            modules.entry(placement.package().crate_name()).or_default().extend(placement.module().segments());
            modules
        });
        let places = files.iter().map(|file| {
            let placement = file.placement();
            let place = Place::new(
                placement.package().crate_name(),
                placement.module().segments(),
                Scope::of_file(file.syntax()),
            );
            (file.path(), place)
        });
        Resolver {
            declaring: members.fold(BTreeMap::new(), |mut declaring: BTreeMap<_, Vec<_>>, member| {
                declaring.entry(member.name()).or_default().push(member);
                declaring
            }),
            modules,
            places: places.collect(),
            stand_ins,
        }
    }

    fn target(&self, file: &Path, name: &Name, written: &[Name], origin: Option<&Name>) -> Option<Target<'found>> {
        let place = self.places.get(file)?;
        Reach::new(place, &self.modules).target(name, written, origin)
    }

    pub(crate) fn members(
        &self,
        file: &Path,
        name: &Name,
        written: &[Name],
        origin: Option<&Name>,
    ) -> Vec<Member<'found>> {
        let declaring = self.declaring.get(name).map_or(&[][..], Vec::as_slice);
        let single = || declaring.first().copied().filter(|_| declaring.len() == 1).into_iter().collect();
        self.target(file, name, written, origin).map_or_else(Vec::new, |target| {
            Some(self.within(&target, name)).filter(|found| !found.is_empty()).unwrap_or_else(single)
        })
    }

    fn within(&self, target: &Target<'found>, name: &Name) -> Vec<Member<'found>> {
        let krate = target.krate();
        let declaring = |name: &Name| self.declaring.get(name).map_or(&[][..], Vec::as_slice);
        let stood = |name: &Name| self.stand_ins.standing_for(krate, name).iter();
        let unplaced = Target::new(krate, None, None);
        target.declared(declaring(name)).map_or_else(
            || {
                let reached = Traversal::trees(stood(name), stood).into_iter().flatten();
                reached.filter_map(|name| unplaced.declared(declaring(name))).collect()
            },
            |member| vec![member],
        )
    }

    pub(crate) fn dependency(&self, dependency: &Dependency<'_>) -> Vec<Member<'found>> {
        let site: &Site<'_> = dependency.as_ref();
        self.members(site.path(), dependency.as_ref(), dependency.written(), None)
    }

    pub(crate) fn implemented(
        &self,
        placed: Member<'found>,
        implementation: &Implementation<'_>,
    ) -> Vec<Member<'found>> {
        let site: &Site<'_> = implementation.as_ref();
        let own = self.members(site.path(), placed.name(), &[], None);
        own.into_iter()
            .filter(|member| *member == placed)
            .flat_map(|_| self.members(site.path(), implementation.as_ref(), &[], None))
            .collect()
    }

    pub(crate) fn endpoint(&self, file: &Path, endpoint: &Endpoint) -> Vec<Member<'found>> {
        match endpoint.nature() {
            Nature::Named => self.members(file, endpoint.as_ref(), &[], endpoint.origin().map(Borrow::borrow)),
            Nature::Interface => self.members(file, endpoint.as_ref(), &[], None),
            Nature::Primitive | Nature::Boxed => Vec::new(),
        }
    }

    pub(crate) fn bodies(&self, file: &SourceFile) -> Vec<Link<'found>> {
        let path = file.path();
        let krate = file.placement().package().crate_name();
        let uses = ReferencedNames::of_file(file, Breadth::Workspace).into_uses();
        uses.iter().flat_map(|(owner, used)| self.body(path, krate, owner, used)).collect()
    }

    fn body(&self, file: &Path, krate: &CrateName, owner: &Name, used: &BTreeSet<UsePath>) -> Vec<Link<'found>> {
        let declaring = self.declaring.get(owner).map_or(&[][..], Vec::as_slice);
        let holder = self.target(file, owner, &[], None).and_then(|target| target.declared(declaring));
        used.iter()
            .flat_map(|path| self.used(file, krate, path))
            .filter_map(|end| holder.map(|holder| Link::new(holder, end)))
            .collect()
    }

    fn used(&self, file: &Path, krate: &CrateName, path: &UsePath) -> Vec<Member<'found>> {
        let named = path.naming(|name| {
            let standing = self.stand_ins.standing_for(krate, name).first().and(Some(name));
            self.declaring.get(name).and(Some(name)).or(standing)
        });
        let found = named.segments().split_last().and_then(|(name, written)| {
            self.target(file, name, written, None).map(|target| self.within(&target, name))
        });
        found.unwrap_or_default()
    }

    pub(crate) fn signature(&self, transition: Transition<'found, '_>) -> Vec<Link<'found>> {
        let site: &Site<'_> = transition.as_ref();
        let file = site.path();
        let holders = transition.context().subject().map_or_else(Vec::new, |subject| self.endpoint(file, subject));
        let ends = transition.takes().iter().chain([transition.gives().endpoint()]);
        ends.flat_map(|end| self.endpoint(file, end))
            .flat_map(|end| holders.iter().map(move |holder| Link::new(*holder, end)))
            .collect()
    }
}
