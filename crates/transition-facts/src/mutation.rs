#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mutation {
    Absent,
    ThroughSelf,
    ThroughParameter,
}
