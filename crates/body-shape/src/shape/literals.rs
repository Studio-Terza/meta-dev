use super::spelling::Spelled;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Literals(Vec<Spelled>);

impl FromIterator<Spelled> for Literals {
    fn from_iter<Found: IntoIterator<Item = Spelled>>(found: Found) -> Literals {
        Literals(found.into_iter().collect())
    }
}

impl AsRef<[Spelled]> for Literals {
    fn as_ref(&self) -> &[Spelled] {
        &self.0
    }
}
