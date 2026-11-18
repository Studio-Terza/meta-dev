mod boundary;

use newtype::{Arbitrary, Display, Newtype};

use crate::matching::{Fit, Matching};
use crate::name::Name;
use crate::word::Word;
use boundary::Letters;

#[derive(Debug)]
pub struct CamelWord;

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hump(Name);

impl Name {
    pub fn humps(&self) -> impl Iterator<Item = Hump> {
        self.words()
            .into_iter()
            .flat_map(|chunk| Letters::humps(&chunk.0))
            .map(|hump| Hump(Name::from(Name(std::sync::Arc::from(hump)).lowered())))
    }
}

impl AsRef<Name> for Hump {
    fn as_ref(&self) -> &Name {
        &self.0
    }
}

impl Matching for CamelWord {
    fn holds(name: &Name, word: Word<'_>) -> Fit {
        Fit::from(name.humps().find(|hump| *hump.as_ref() == Name::from(word)))
    }
}
