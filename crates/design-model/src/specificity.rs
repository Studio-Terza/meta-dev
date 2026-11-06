use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Specificity {
    #[name("general")]
    General,
    #[name("specific")]
    Specific,
    #[name("exact")]
    Exact,
}
