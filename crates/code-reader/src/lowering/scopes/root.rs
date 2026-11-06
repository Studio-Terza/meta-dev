use code_model::{Resolved, TraitPath};
use lint_layout::{Name, Whole, Word, words};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Root {
    Std,
    Crate,
    Scoped,
}

impl Root {
    const STD: Word<'static> = Word::new("std");
    pub(super) const CRATE: Word<'static> = Word::new("crate");
    const STD_ROOTS: [Word<'static>; 3] = words!["std", "core", "alloc"];
    const CRATE_ROOTS: [Word<'static>; 3] = words!["crate", "self", "super"];
    pub(super) const PRELUDE: [[Word<'static>; 3]; 9] = [
        words!["std", "clone", "Clone"],
        words!["std", "marker", "Copy"],
        words!["std", "fmt", "Debug"],
        words!["std", "default", "Default"],
        words!["std", "cmp", "Eq"],
        words!["std", "cmp", "PartialEq"],
        words!["std", "cmp", "Ord"],
        words!["std", "cmp", "PartialOrd"],
        words!["std", "hash", "Hash"],
    ];

    pub(super) fn rooted(segments: &[Name]) -> Resolved {
        match segments.split_first().map(|(first, rest)| (Root::from(first), rest)) {
            Some((Root::Std, rest)) => Resolved::Std(Self::prefixed(Root::STD, rest)),
            Some((Root::Crate, rest)) => Resolved::Crate(Self::prefixed(Root::CRATE, rest)),
            Some((Root::Scoped, _)) | None => Resolved::External(segments.iter().cloned().collect()),
        }
    }

    pub(super) fn prefixed(root: Word<'_>, rest: &[Name]) -> TraitPath {
        std::iter::once(Name::from(root)).chain(rest.iter().cloned()).collect()
    }
}

impl From<&Name> for Root {
    fn from(first: &Name) -> Root {
        first
            .matches::<Whole>(&Root::STD_ROOTS)
            .map(|_| Root::Std)
            .or_else(|| first.matches::<Whole>(&Root::CRATE_ROOTS).map(|_| Root::Crate))
            .unwrap_or(Root::Scoped)
    }
}
