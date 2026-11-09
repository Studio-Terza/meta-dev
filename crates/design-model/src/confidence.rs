use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    #[name("tentative")]
    Tentative,
    #[name("leaning")]
    Leaning,
    #[name("firm")]
    Firm,
}
