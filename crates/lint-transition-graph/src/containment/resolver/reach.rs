use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;

mod head;

use super::place::Place;
use super::target::Target;
use head::Head;

#[derive(Debug)]
pub(crate) struct Reach<'place, 'found> {
    place: &'place Place<'found>,
    modules: &'place BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
}

impl<'place, 'found> Reach<'place, 'found> {
    const EXTERNAL: [Word<'static>; 3] = words!["std", "core", "alloc"];

    pub(crate) fn new(
        place: &'place Place<'found>,
        modules: &'place BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
    ) -> Reach<'place, 'found> {
        Reach { place, modules }
    }

    pub(crate) fn target(&self, name: &Name, written: &[Name], origin: Option<&Name>) -> Option<Target<'found>> {
        let spelled = self.place.scope().path_to(name, written);
        spelled.first().map_or_else(
            || {
                origin.map_or_else(
                    || Some(self.place.target(None)),
                    |origin| self.pathed(std::slice::from_ref(origin)).map(Target::unhinted),
                )
            },
            |_| self.pathed(&spelled),
        )
    }

    fn pathed(&self, path: &[Name]) -> Option<Target<'found>> {
        let (head, _) = path.split_first()?;
        self.head_of(head).target(self.place, path)
    }

    fn head_of(&self, head: &Name) -> Head<'found> {
        Head::KEYWORDS
            .into_iter()
            .find(|(keyword, _)| *head == Name::from(*keyword))
            .map(|(_, kind)| kind)
            .or_else(|| self.place.scope().module_root(head).map(|_| Head::Local))
            .or_else(|| {
                self.modules.keys().copied().find(|krate| Borrow::<Name>::borrow(*krate) == head).map(Head::Krate)
            })
            .or_else(|| head.matches::<Whole>(&Self::EXTERNAL).map(|_| Head::External))
            .unwrap_or_else(|| {
                self.modules
                    .get(self.place.krate())
                    .and_then(|segments| segments.get(head))
                    .map_or(Head::Unknown, |_| Head::Own)
            })
    }
}
