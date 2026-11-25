use newtype::{Arbitrary, Counter, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Holes(usize);

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Differing(usize);

impl Holes {
    #[must_use]
    pub fn between<Item: PartialEq>(sequences: &[&[Item]]) -> Holes {
        let first = sequences.first().copied().unwrap_or_default();
        let longest = sequences.iter().map(|sequence| sequence.len()).max().unwrap_or_default();
        Holes::new(
            (0..longest).filter(|at| sequences.iter().any(|sequence| sequence.get(*at) != first.get(*at))).count(),
        )
    }

    #[must_use]
    pub fn differing(self) -> Option<Differing> {
        Some(self).filter(|holes| *holes != Holes::zero()).map(|holes| Differing::new(holes.count()))
    }
}
