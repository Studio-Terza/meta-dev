use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeRelation {
    #[name("refines")]
    Refines,
    #[name("consumes")]
    Consumes,
}
