use std::borrow::Borrow;
use std::collections::{BTreeMap, BTreeSet};

use lint_layout::{Name, Whole, Word, words};
use lint_placement::CrateName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reach<'found> {
    Local,
    Crate(&'found CrateName),
    External,
}

impl<'found> Reach<'found> {
    const EXTERNAL: [Word<'static>; 3] = words!["std", "core", "alloc"];
    const LOCAL: [Word<'static>; 3] = words!["crate", "self", "super"];

    pub(crate) fn of_hint(
        hint: Option<&Name>,
        from: &CrateName,
        modules: &BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
    ) -> Reach<'found> {
        hint.map_or(Reach::Local, |named| Self::of_named(named, from, modules))
    }

    pub(crate) fn krate<'reached>(self, from: &'reached CrateName) -> Option<&'reached CrateName>
    where
        'found: 'reached,
    {
        match self {
            Reach::Local => Some(from),
            Reach::Crate(krate) => Some(krate),
            Reach::External => None,
        }
    }

    fn of_named(
        named: &Name,
        from: &CrateName,
        modules: &BTreeMap<&'found CrateName, BTreeSet<&'found Name>>,
    ) -> Reach<'found> {
        let krate = modules.keys().copied().find(|krate| Borrow::<Name>::borrow(*krate) == named);
        let external = named.matches::<Whole>(&Self::EXTERNAL);
        let root = named.matches::<Whole>(&Self::LOCAL);
        let module = modules.get(from).and_then(|segments| segments.get(named));
        match (krate, external, root, module) {
            (Some(krate), _, _, _) => Reach::Crate(krate),
            (None, None, Some(_), _) | (None, None, None, Some(_)) => Reach::Local,
            (None, _, _, _) => Reach::External,
        }
    }
}
