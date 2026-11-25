use newtype::Arbitrary;

use crate::name::Name;
use crate::word::Word;

#[derive(Arbitrary, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[newtype(from = PackageName)]
pub struct PackageName(Name);

impl From<&Name> for PackageName {
    fn from(name: &Name) -> PackageName {
        PackageName(name.clone())
    }
}

impl From<Word<'_>> for PackageName {
    fn from(word: Word<'_>) -> PackageName {
        PackageName(Name::from(word))
    }
}
