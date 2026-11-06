use design_store::Earlier;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ingested,
    Shown,
    Searched,
    Initialised,
    Applied,
    AlreadyApplied { earlier: Earlier },
}
