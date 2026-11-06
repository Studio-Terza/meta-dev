#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Ingested,
    Shown,
    Searched,
    Initialised,
    Applied,
}
