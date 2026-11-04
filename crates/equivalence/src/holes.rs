use newtype::{Arbitrary, Counter, Display, Newtype};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Holes(usize);

impl Holes {
    #[must_use]
    pub fn between<Item: PartialEq>(sequences: &[&[Item]]) -> Holes {
        let first = sequences.first().copied().unwrap_or_default();
        Holes::new(
            first
                .iter()
                .enumerate()
                .filter(|(at, item)| sequences.iter().any(|sequence| sequence.get(*at) != Some(*item)))
                .count(),
        )
    }

    #[must_use]
    pub fn none(self) -> Option<Holes> {
        Some(self).filter(|holes| *holes == Holes::zero())
    }
}
