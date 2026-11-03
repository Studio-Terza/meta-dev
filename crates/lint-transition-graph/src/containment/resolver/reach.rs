use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;

use super::place::Place;
use super::target::Target;

#[derive(Debug)]
pub(crate) struct Reach<'place, 'found> {
    place: &'place Place<'found>,
    modules: &'place BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
}

impl<'place, 'found> Reach<'place, 'found> {
    const EXTERNAL: [Word<'static>; 3] = words!["std", "core", "alloc"];
    const ROOT: Word<'static> = Word::new("crate");
    const ITSELF: Word<'static> = Word::new("self");
    const PARENT: Word<'static> = Word::new("super");

    pub(crate) fn new(
        place: &'place Place<'found>,
        modules: &'place BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
    ) -> Reach<'place, 'found> {
        Reach { place, modules }
    }

    pub(crate) fn target(&self, name: &Name, written: &[Name], origin: Option<&Name>) -> Option<Target<'found>> {
        let spelled = self.place.scope().path_to(name, written);
        let path = match spelled.first() {
            Some(_) => spelled,
            None => origin.into_iter().cloned().collect(),
        };
        path.first().map_or_else(|| Some(self.place.target(None)), |_| self.pathed(&path))
    }

    fn pathed(&self, path: &[Name]) -> Option<Target<'found>> {
        let (head, rest) = path.split_first()?;
        let here = self.place.module();
        let parents = path.iter().take_while(|step| **step == Self::PARENT).count();
        let within = |base: &[Name], tail: &[Name]| base.iter().chain(tail).cloned().collect::<Vec<Name>>();
        let krate = self.modules.keys().copied().find(|krate| Borrow::<Name>::borrow(*krate) == head);
        let own = self.modules.get(self.place.krate()).and_then(|segments| segments.get(head));
        match (head, krate, self.place.scope().module_root(head)) {
            (root, _, _) if *root == Self::ROOT => Some(self.place.target(Some(rest.to_vec()))),
            (itself, _, _) if *itself == Self::ITSELF => Some(self.place.target(Some(within(here, rest)))),
            (parent, _, _) if *parent == Self::PARENT => {
                let base = here.get(..here.len().saturating_sub(parents)).unwrap_or_default();
                Some(self.place.target(Some(within(base, path.get(parents..).unwrap_or_default()))))
            }
            (_, _, Some(_)) => Some(self.place.target(Some(within(here, path)))),
            (_, Some(krate), None) if krate == self.place.krate() => Some(self.place.target(Some(rest.to_vec()))),
            (_, Some(krate), None) => Some(Target::new(krate, Some(rest.to_vec()), None)),
            (_, None, None) => {
                head.matches::<Whole>(&Self::EXTERNAL).map_or(own.map(|_| self.place.target(None)), |_| None)
            }
        }
    }
}
