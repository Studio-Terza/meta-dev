use named_derive::Named;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    Tentative,
    Leaning,
    Firm,
}
