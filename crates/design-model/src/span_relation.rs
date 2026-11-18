use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanRelation {
    Defines,
    Decides,
    Questions,
    Revises,
    Mentions,
}
