use design_store::Earlier;
use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jobs(usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ingested,
    Shown,
    Searched,
    Initialised,
    Applied,
    AlreadyApplied { earlier: Earlier },
    Structured(Jobs),
}
