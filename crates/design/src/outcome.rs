use design_model::{Role, Stage};
use design_store::Earlier;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ingested,
    Shown,
    Searched,
    Initialised,
    Applied,
    AlreadyApplied { earlier: Earlier },
    Moved(Stage),
    Recast(Role),
}
