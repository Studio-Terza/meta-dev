mod step;

use std::collections::BTreeSet;

use lint_core::{Least, Site};
use lint_layout::{Length, Name, Whole, Word, words};
use type_facts::{Shape, TypeFact};

use step::Step;

#[derive(Debug)]
pub(crate) struct Walk<'facts, 'file> {
    found: &'facts TypeFact<'file>,
    steps: BTreeSet<Step>,
}

impl<'facts, 'file> Walk<'facts, 'file> {
    const EXCLUDED_DERIVE: [Word<'static>; 1] = words!["Newtype"];
    const SMALLEST: Least = Least::new(4);

    pub(crate) fn walked(found: &'facts TypeFact<'file>, local: &BTreeSet<Name>) -> Option<Walk<'facts, 'file>> {
        let steps: BTreeSet<Step> = Step::walked_by(found, local).into_iter().collect();
        Self::SMALLEST
            .allows(Length::new(steps.len()))
            .filter(|_| found.shape() == Shape::Other)
            .filter(|_| found.derives().iter().all(|name| name.matches::<Whole>(&Self::EXCLUDED_DERIVE).is_none()))
            .map(|_| Walk { found, steps })
    }

    pub(crate) fn steps(&self) -> &BTreeSet<Step> {
        &self.steps
    }
}

impl<'file> AsRef<Site<'file>> for Walk<'_, 'file> {
    fn as_ref(&self) -> &Site<'file> {
        self.found.as_ref()
    }
}

impl AsRef<Name> for Walk<'_, '_> {
    fn as_ref(&self) -> &Name {
        self.found.as_ref()
    }
}
