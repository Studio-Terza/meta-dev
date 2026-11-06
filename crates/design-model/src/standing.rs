use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    #[name("current")]
    Current,
    #[name("superseded")]
    Superseded,
}
