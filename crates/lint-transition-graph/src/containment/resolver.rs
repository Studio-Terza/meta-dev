mod functions;
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
use transition_facts::{Endpoint, Nature, Transition};
use type_facts::{Dependency, Implementation};

use super::housing::Housing;
use super::link::Link;
use super::member::Member;
use super::unit::Unit;
pub(crate) use functions::Functions;
use place::Place;
use reach::Reach;
pub(crate) use stand_ins::StandIns;
use target::Target;

#[derive(Debug, Default)]
pub(crate) struct Resolver<'found> {
    declaring: BTreeMap<&'found Name, Vec<Member<'found>>>,
    modules: BTreeMap<Unit<'found>, BTreeSet<&'found Name>>,
    places: HashMap<&'found Path, Place<'found>>,
    stand_ins: StandIns<'found>,
}

impl<'found> Resolver<'found> {
    pub(crate) fn of_members(
        members: impl Iterator<Item = Member<'found>>,
        functions: &Functions<'found>,
        housing: &Housing<'found>,
        stand_ins: StandIns<'found>,
    ) -> Resolver<'found> {
        let homed: Vec<(&'found SourceFile, Unit<'found>)> =
            housing.files().iter().filter_map(|file| housing.unit(file.path()).map(|unit| (*file, unit))).collect();
        let modules = homed.iter().fold(BTreeMap::new(), |mut modules: BTreeMap<_, BTreeSet<_>>, (file, unit)| {
            modules.entry(*unit).or_default().extend(file.placement().module().segments());
            modules
        });
        let places = homed.iter().map(|(file, unit)| {
            let place = Place::new(
                *unit,
                file.placement().module().segments(),
                Scope::of_file(file.syntax()),
                functions.in_file(file.path()),
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
        let unit = target.unit();
        let declaring = |name: &Name| self.declaring.get(name).map_or(&[][..], Vec::as_slice);
        let stood = |name: &Name| self.stand_ins.standing_for(unit, name).iter();
        let unplaced = Target::new(unit, None, None);
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
        self.places
            .get(file.path())
            .map_or_else(Vec::new, |place| self.bodies_reached(&Reach::new(place, &self.modules), place.unit(), names))
    }

    fn bodies_reached(
        &self,
        reach: &Reach<'_, 'found>,
        unit: Unit<'found>,
        names: &ReferencedNames,
    ) -> Vec<Link<'found>> {
        let uses = names.uses();
        let found: BTreeMap<&UsePath, Vec<Member<'found>>> =
            uses.values().flatten().map(|path| (*path, self.used(reach, unit, path))).collect();
        uses.iter()
            .filter_map(|(owner, used)| {
                let holder = reach.holder(owner, |name| self.declaring.get(name).map_or(&[][..], Vec::as_slice))?;
                let ends = used.iter().flat_map(|path| found.get(path).into_iter().flatten());
                Some(ends.map(move |end| Link::held(unit, holder, *end)))
            })
            .flatten()
            .collect()
    }

    fn used(&self, reach: &Reach<'_, 'found>, unit: Unit<'found>, path: &UsePath) -> Vec<Member<'found>> {
        let named = path.naming(|name| {
            let standing = self.stand_ins.standing_for(unit, name).first().and(Some(name));
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
        let holders = transition.context().subject().map_or_else(Vec::new, |subject| self.endpoint(file, subject));
        let ends: Vec<Member<'found>> = transition
            .takes()
            .iter()
            .chain([transition.gives().endpoint()])
            .chain(transition.fails())
            .flat_map(|end| self.endpoint(file, end))
            .collect();
        let own = self.places.get(file).map(Place::unit);
        own.map_or_else(Vec::new, |own| Link::held_between(own, &holders, &ends))
    }
}
