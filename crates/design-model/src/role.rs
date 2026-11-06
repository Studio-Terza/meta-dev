use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    #[name("core")]
    Core,
    #[name("secondary")]
    Secondary,
}
