use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Calls(BTreeSet<Name>);

impl Calls {
    pub(crate) fn naming(names: impl IntoIterator<Item = Name>) -> Calls {
        Calls(names.into_iter().collect())
    }
}

impl PartialOrd for Calls {
    fn partial_cmp(&self, other: &Calls) -> Option<Ordering> {
        let above = other.0.is_subset(&self.0).then_some(Ordering::Greater);
        let below = self.0.is_subset(&other.0).then_some(Ordering::Less);
        above.zip(below).map(|_| Ordering::Equal).or(above).or(below)
    }
}

impl fmt::Display for Calls {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|name| write!(formatter, " calling {name}"))
    }
}
