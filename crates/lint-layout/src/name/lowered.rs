use crate::name::Name;

#[derive(Debug, PartialEq, Eq)]
pub struct Lowered(String);

impl FromIterator<char> for Lowered {
    fn from_iter<Letters: IntoIterator<Item = char>>(letters: Letters) -> Lowered {
        Lowered(letters.into_iter().collect())
    }
}

impl From<Lowered> for Name {
    fn from(lowered: Lowered) -> Name {
        Name(lowered.0)
    }
}
