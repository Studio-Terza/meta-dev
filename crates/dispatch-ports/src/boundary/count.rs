use newtype::{Arbitrary, Deserialize, Newtype};

#[derive(Newtype, Arbitrary, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Count(u32);

impl Count {
    pub(crate) fn reached(self, seen: usize) -> Option<()> {
        (usize::try_from(self.0).is_ok_and(|wanted| wanted == seen)).then_some(())
    }

    pub(crate) fn counted(self) -> Count {
        Count(self.0.saturating_add(1))
    }
}
