mod boundary;

use newtype::{Arbitrary, Display, Newtype};

use crate::name::Name;
use boundary::Letters;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hump(Name);

impl Name {
    pub fn humps(&self) -> impl Iterator<Item = Hump> {
        self.words()
            .into_iter()
            .flat_map(|chunk| Letters::humps(&chunk.0))
            .map(|hump| Hump(Name::from(Name(hump).lowered())))
    }
}

impl AsRef<Name> for Hump {
    fn as_ref(&self) -> &Name {
        &self.0
    }
}
