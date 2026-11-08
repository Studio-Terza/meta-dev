use crate::name::Name;
use crate::word::Word;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
