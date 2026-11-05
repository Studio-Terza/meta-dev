mod place;
mod reach;
mod stand_ins;
mod target;

use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use lint_core::{Site, SourceFile};
use lint_item_graph::{ReferencedNames, Scope, Traversal};
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
    places: HashMap<&'found Path, Place<'found>>,
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

    pub(crate) fn members(
        &self,
        file: &Path,
        name: &Name,
        written: &[Name],
        origin: Option<&Name>,
    ) -> Vec<Member<'found>> {
        self.places
            .get(file)
            .map_or_else(Vec::new, |place| self.reached(&Reach::new(place, &self.modules), name, written, origin))
    }

    fn reached(
        &self,
        reach: &Reach<'_, 'found>,
        name: &Name,
        written: &[Name],
        origin: Option<&Name>,
    ) -> Vec<Member<'found>> {
        let declaring = self.declaring.get(name).map_or(&[][..], Vec::as_slice);
        let single = || declaring.first().copied().filter(|_| declaring.len() == 1).into_iter().collect();
        reach.target(name, written, origin).map_or_else(Vec::new, |target| {
            let found = self.within(&target, name);
            found.first().map(|_| ()).map_or_else(single, |()| found)
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
            .flat_map(|_| self.members(site.path(), implementation.as_ref(), implementation.written(), None))
            .collect()
    }

    pub(crate) fn argument(&self, implementation: &Implementation<'_>) -> Vec<Member<'found>> {
        let site: &Site<'_> = implementation.as_ref();
        implementation.argument().map_or_else(Vec::new, |argument| self.members(site.path(), argument, &[], None))
    }

    pub(crate) fn endpoint(&self, file: &Path, endpoint: &Endpoint) -> Vec<Member<'found>> {
        match endpoint.nature() {
            Nature::Named => self.members(file, endpoint.as_ref(), &[], endpoint.origin().map(Borrow::borrow)),
            Nature::Interface => self.members(file, endpoint.as_ref(), &[], None),
            Nature::Primitive | Nature::Boxed => Vec::new(),
        }
    }

    pub(crate) fn bodies(&self, file: &SourceFile, names: &ReferencedNames) -> Vec<Link<'found>> {
        let krate = file.placement().package().crate_name();
        self.places
            .get(file.path())
            .map_or_else(Vec::new, |place| self.bodies_reached(&Reach::new(place, &self.modules), krate, names))
    }

    fn bodies_reached(
        &self,
        reach: &Reach<'_, 'found>,
        krate: &CrateName,
        names: &ReferencedNames,
    ) -> Vec<Link<'found>> {
        let uses = names.uses();
        let found: BTreeMap<&UsePath, Vec<Member<'found>>> =
            uses.values().flatten().map(|path| (*path, self.used(reach, krate, path))).collect();
        uses.iter()
            .filter_map(|(owner, used)| {
                let holder = reach.holder(owner, |name| self.declaring.get(name).map_or(&[][..], Vec::as_slice))?;
                let ends = used.iter().flat_map(|path| found.get(path).into_iter().flatten());
                Some(ends.map(move |end| Link::held(krate, holder, *end)))
            })
            .flatten()
            .collect()
    }

    fn used(&self, reach: &Reach<'_, 'found>, krate: &CrateName, path: &UsePath) -> Vec<Member<'found>> {
        let named = path.naming(|name| {
            let standing = self.stand_ins.standing_for(krate, name).first().and(Some(name));
            self.declaring.get(name).and(Some(name)).or(standing)
        });
        let found = named
            .segments()
            .split_last()
            .and_then(|(name, written)| reach.target(name, written, None).map(|target| self.within(&target, name)));
        found.unwrap_or_default()
    }

    pub(crate) fn signature(&self, transition: Transition<'found, '_>) -> Vec<Link<'found>> {
        let site: &Site<'_> = transition.as_ref();
        let file = site.path();
        let own: &CrateName = transition.as_ref();
        let holders = transition.context().subject().map_or_else(Vec::new, |subject| self.endpoint(file, subject));
        let ends: Vec<Member<'found>> = transition
            .takes()
            .iter()
            .chain([transition.gives().endpoint()])
            .chain(transition.fails())
            .flat_map(|end| self.endpoint(file, end))
            .collect();
        holders.iter().flat_map(|holder| ends.iter().map(move |end| Link::held(own, *holder, *end))).collect()
    }
}
