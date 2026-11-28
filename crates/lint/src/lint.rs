use std::sync::Arc;

use pipeline::Measure;

use crate::finding::{Finding, Findings};
use crate::gate::Gate;
use crate::hooks::Hooks;
use crate::source::{Place, Source};
use crate::workspace::Package;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reads {
    All,
    Every,
    Ungated,
    Shipped,
}

pub(crate) trait Lint<'file>: Hooks + Sized {
    const HEADLINE: &'static str;
    const READS: Reads;

    type Fact: Fact;

    fn new(source: &'file Source) -> Self;

    fn fact(self, source: &'file Source) -> Self::Fact;
}

pub(crate) trait Fact: Measure + Send {
    fn findings(self, packages: &[Arc<Package>]) -> Vec<Finding>;
}

impl Reads {
    pub(crate) fn admits(self, source: &Source) -> Option<()> {
        match self {
            Reads::All => Some(()),
            Reads::Every | Reads::Ungated => source.gate().entered(),
            Reads::Shipped => source.gate().entered().filter(|()| source.place() == Place::Shipped),
        }
    }

    pub(crate) fn skips(self) -> Option<()> {
        match self {
            Reads::All | Reads::Every => None,
            Reads::Ungated | Reads::Shipped => Some(()),
        }
    }

    pub(crate) fn skipped(gate: Gate) -> Option<()> {
        gate.entered().xor(Some(()))
    }
}

impl Fact for Findings {
    fn findings(self, _: &[Arc<Package>]) -> Vec<Finding> {
        self.into_iter().collect()
    }
}

macro_rules! style {
    ($lint:ident, $reads:ident, $headline:literal) => {
        impl<'file> $crate::lint::Lint<'file> for $lint {
            const HEADLINE: &'static str = $headline;
            const READS: $crate::lint::Reads = $crate::lint::Reads::$reads;

            type Fact = $crate::finding::Findings;

            fn new(_: &'file $crate::source::Source) -> $lint {
                $lint::default()
            }

            fn fact(self, source: &'file $crate::source::Source) -> $crate::finding::Findings {
                $crate::finding::Findings::found($headline, source.shown(), self.found)
            }
        }
    };
}

pub(crate) use style;
