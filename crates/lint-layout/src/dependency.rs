use newtype::proptest::arbitrary::{Arbitrary, any};
use newtype::proptest::strategy::{BoxedStrategy, Strategy};

use crate::line_number::LineNumber;
use crate::name::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dependency {
    name: Name,
    line: LineNumber,
}

impl Dependency {
    pub(crate) fn new(name: Name, line: LineNumber) -> Dependency {
        Dependency { name, line }
    }

    #[must_use]
    pub fn line(&self) -> LineNumber {
        self.line
    }
}

impl AsRef<Name> for Dependency {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl Arbitrary for Dependency {
    type Parameters = ();
    type Strategy = BoxedStrategy<Dependency>;

    fn arbitrary_with((): ()) -> BoxedStrategy<Dependency> {
        (any::<Name>(), any::<usize>()).prop_map(|(name, line)| Dependency::new(name, LineNumber::from(line))).boxed()
    }
}
