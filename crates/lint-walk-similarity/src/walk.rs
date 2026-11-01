mod step;

use std::collections::BTreeSet;

use lint_core::{Least, Site};
use lint_layout::{Name, Whole, Word, words};
use type_facts::TypeFact;

use step::Step;

#[derive(Debug)]
pub(crate) struct Walk<'facts> {
    found: &'facts TypeFact,
    steps: BTreeSet<Step>,
}

impl<'facts> Walk<'facts> {
    const EXCLUDED_DERIVE: [Word<'static>; 1] = words!["Newtype"];
    const SMALLEST: Least = Least::new(4);

    pub(crate) fn walked(found: &'facts TypeFact) -> Option<Walk<'facts>> {
        let steps: BTreeSet<Step> = Step::walked_by(found).collect();
        (!found.derives().iter().any(|name| name.matches::<Whole>(&Self::EXCLUDED_DERIVE))
            && Self::SMALLEST.allows(steps.len()))
        .then_some(Walk { found, steps })
    }

    pub(crate) fn steps(&self) -> &BTreeSet<Step> {
        &self.steps
    }
}

impl AsRef<Site> for Walk<'_> {
    fn as_ref(&self) -> &Site {
        self.found.as_ref()
    }
}

impl AsRef<Name> for Walk<'_> {
    fn as_ref(&self) -> &Name {
        self.found.as_ref()
    }
}
