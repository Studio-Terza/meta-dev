use named_derive::Named;
use strum::VariantArray;

#[derive(Named, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    #[name("vague")]
    Vague,
    #[name("refining")]
    Refining,
    #[name("terminal")]
    Terminal,
    #[name("dropped")]
    Dropped,
}
