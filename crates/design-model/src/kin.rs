use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kin {
    #[name("ancestor")]
    Ancestor,
    #[name("child")]
    Child,
}
