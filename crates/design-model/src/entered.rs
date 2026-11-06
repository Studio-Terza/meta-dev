use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entered {
    #[name("init")]
    Init,
    #[name("apply")]
    Apply,
    #[name("role")]
    Role,
    #[name("inheritance")]
    Inheritance,
}
