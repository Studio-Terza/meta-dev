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

    pub fn judged(written: &UsePath, candidates: &[UsePath], ports: &PortTraits) -> Kind {
        candidates
            .iter()
            .map(|candidate| Self::resolved(candidate, ports))
            .chain(iter::once(Self::bare(written)))
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
        match modules.first() {
            _ if modules.iter().any(|module| *module == Self::PORTS) => Kind::Port,
            Some(first) if first.words().last().is_some_and(|word| *word == Self::PORTS) => Kind::Port,
            Some(first) if ports.declares(first, last) => Kind::Port,
            Some(first) if first.matches::<Whole>(&Self::STD) => Kind::Std,
            _ => Kind::Other,
        }
    }

    fn bare(written: &UsePath) -> Kind {
        match written.segments() {
            [only] if only.matches::<Whole>(&Self::PRELUDE) => Kind::Std,
            _ => Kind::Other,
        }
    }
}
