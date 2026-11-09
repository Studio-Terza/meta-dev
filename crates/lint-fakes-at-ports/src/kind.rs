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
    pub fn written(written: &UsePath) -> Kind {
        written
            .segments()
            .split_first()
            .filter(|(_, rest)| rest.is_empty())
            .and_then(|(only, _)| only.matches::<Whole>(&Self::PRELUDE))
            .map_or(Kind::Other, |_| Kind::Std)
    }

    #[must_use]
    pub fn judged(self, candidates: &[UsePath], ports: &PortTraits) -> Kind {
        candidates
            .iter()
            .map(|candidate| Self::resolved(candidate, ports))
            .chain(iter::once(self))
            .min()
            .unwrap_or(Kind::Other)
    }

    fn resolved(candidate: &UsePath, ports: &PortTraits) -> Kind {
        match candidate.segments().split_last() {
            Some((last, modules)) => Self::within(last, modules, ports),
            None => Kind::Other,
        }
    }

    fn within(last: &Name, modules: &[Name], ports: &PortTraits) -> Kind {
        let first = modules.first();
        let named = modules.iter().find(|module| **module == Name::from(Self::PORTS)).map(|_| Kind::Port);
        let suffixed = first
            .filter(|module| module.words().last().is_some_and(|word| *word == Name::from(Self::PORTS)))
            .map(|_| Kind::Port);
        let declared = first.filter(|module| ports.declares(module, last).is_some()).map(|_| Kind::Port);
        let standard = first.filter(|module| module.matches::<Whole>(&Self::STD).is_some()).map(|_| Kind::Std);
        named.or(suffixed).or(declared).or(standard).unwrap_or(Kind::Other)
    }
}
