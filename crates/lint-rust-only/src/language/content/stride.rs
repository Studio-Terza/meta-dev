use std::iter::StepBy;

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug)]
pub(super) struct Stride(usize);

impl Stride {
    pub(super) fn every_nth<Items: Iterator>(self, items: Items) -> StepBy<Items> {
        items.step_by(self.0)
    }
}
