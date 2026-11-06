use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Explicitness {
    #[name("explicit")]
    Explicit,
    #[name("implicit")]
    Implicit,
}
