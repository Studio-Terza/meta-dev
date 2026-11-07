use std::iter;

use lint_layout::{Name, UsePath, Whole, Word, words};

use crate::port_traits::PortTraits;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Port,
    Std,
    Other,
}

impl Kind {
    const PORTS: &str = "ports";
    const STD: [Word<'static>; 3] = words!["std", "core", "alloc"];
    const PRELUDE: [Word<'static>; 27] = words![
        "AsMut",
        "AsRef",
        "Clone",
        "Copy",
        "Default",
        "DoubleEndedIterator",
        "Drop",
        "Eq",
        "ExactSizeIterator",
        "Extend",
        "Fn",
        "FnMut",
        "FnOnce",
        "From",
        "FromIterator",
        "Into",
        "IntoIterator",
        "Iterator",
        "Ord",
        "PartialEq",
        "PartialOrd",
        "Send",
        "Sized",
        "Sync",
        "ToOwned",
        "ToString",
        "Unpin",
    ];

    #[must_use]
    pub fn written(written: &UsePath) -> Self {
        written
            .segments()
            .split_first()
            .filter(|(_, rest)| rest.is_empty())
            .and_then(|(only, _)| only.matches::<Whole>(&Self::PRELUDE))
            .map_or(Self::Other, |_| Self::Std)
    }

    #[must_use]
    pub fn judged(self, candidates: &[UsePath], ports: &PortTraits) -> Self {
        candidates
            .iter()
            .map(|candidate| Self::resolved(candidate, ports))
            .chain(iter::once(self))
            .min()
            .unwrap_or(Self::Other)
    }

    fn resolved(candidate: &UsePath, ports: &PortTraits) -> Self {
        match candidate.segments().split_last() {
            Some((last, modules)) => Self::within(last, modules, ports),
            None => Self::Other,
        }
    }

    fn within(last: &Name, modules: &[Name], ports: &PortTraits) -> Self {
        let first = modules.first();
        let named = modules.iter().find(|module| **module == Name::from(Self::PORTS)).map(|_| Self::Port);
        let suffixed = first
            .filter(|first| first.words().last().is_some_and(|word| *word == Name::from(Self::PORTS)))
            .map(|_| Self::Port);
        let declared = first.filter(|first| ports.declares(first, last).is_some()).map(|_| Self::Port);
        let standard = first.filter(|first| first.matches::<Whole>(&Self::STD).is_some()).map(|_| Self::Std);
        named.or(suffixed).or(declared).or(standard).unwrap_or(Self::Other)
    }
}
