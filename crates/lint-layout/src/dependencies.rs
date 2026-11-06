mod boundary;

use newtype::{Arbitrary, Newtype};

use crate::dependency::Dependency;

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dependencies(Vec<Dependency>);

impl AsRef<[Dependency]> for Dependencies {
    fn as_ref(&self) -> &[Dependency] {
        &self.0
    }
}
