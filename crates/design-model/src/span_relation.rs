use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanRelation {
    #[name("defines")]
    Defines,
    #[name("decides")]
    Decides,
    #[name("questions")]
    Questions,
    #[name("revises")]
    Revises,
    #[name("mentions")]
    Mentions,
}
