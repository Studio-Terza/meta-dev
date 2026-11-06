use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    #[name("tentative")]
    Tentative,
    #[name("leaning")]
    Leaning,
    #[name("firm")]
    Firm,
}
